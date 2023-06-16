use std::fmt::Display;

use anyhow::Result;
use thiserror::Error;
use tui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};

use crate::{
    fen::Fen,
    piece::{CastleRights, CastleRigthsMask, Piece, PieceError},
};

#[derive(Clone, Copy, Error, Debug)]
pub enum MoveError {
    #[error("tried to move a piece in the wrong turn")]
    WrongTurn,

    #[error("tried to drop piece with no piece grabbed")]
    NoPieceGrabbed,

    #[error("tried to make an illegal move: {mov:?}")]
    IllegalMove { mov: Move },
}

#[derive(Clone, Copy, Error, Debug)]
pub enum BoardError {
    #[error("tried to access a square out of bounds")]
    OutOfBounds,
}

#[derive(Clone, Debug)]
pub struct BoardState {
    pub board: [u8; 64],
    pub white_to_move: bool,
    pub grabbed_piece: Option<u8>,
    pub last_move: Option<Move>,
    pub threatmap: [u8; 64],
    pub castling: CastleRights,
    pub history: Vec<Move>,
}

impl BoardState {
    pub fn from_fen(value: String) -> Result<Self> {
        let fen = Fen::parse(value)?;
        let mut state = BoardState {
            board: fen.board,
            white_to_move: fen.white_to_move,
            grabbed_piece: None,
            last_move: None,
            castling: fen.castling,
            threatmap: [0; 64],
            history: Vec::new(),
        };
        state.update_threatmap();
        Ok(state)
    }

    pub fn as_fen(&self) -> String {
        Fen {
            board: self.board,
            white_to_move: self.white_to_move,
            castling: self.castling,
        }
        .to_string()
    }

    fn in_bounds(&self, ix: u8) -> bool {
        ix < 64
    }

    pub fn make_move(&mut self, mov: Move) -> Result<()> {
        if mov.from == mov.to {
            return Ok(()); // TODO: Change to an error
        }
        if !self.is_legal(&mov) {
            return Err(MoveError::IllegalMove { mov }.into());
        };
        self.add_to_history(mov)?;
        self.move_piece(mov);
        if let Some(sm) = mov.castling {
            self.move_piece(Move::new(sm.0, sm.1));
        }
        self.update_castling_rights(&mov);
        self.pass_turn();
        Ok(())
    }

    fn add_to_history(&mut self, mut mov: Move) -> Result<()> {
        let piece = Piece::try_from(self.board[mov.from.as_ix() as usize])?;
        mov.set_piece(piece);
        self.history.push(mov);
        Ok(())
    }

    fn update_threatmap(&mut self) {
        self.white_to_move = !self.white_to_move;
        self.threatmap = [0; 64];
        for mov in self.get_all_moves() {
            self.threatmap[mov.to.as_ix() as usize] = 1;
        }
        self.white_to_move = !self.white_to_move;
    }

    fn update_castling_rights(&mut self, mov: &Move) {
        // check mov.to as move should already been made
        let piece = Piece::try_from(self.board[mov.to.as_ix() as usize]);
        let (ks_right, qs_right, ks_rook, qs_rook) = match self.white_to_move {
            true => (
                CastleRigthsMask::WhiteKingside,
                CastleRigthsMask::WhiteQueenside,
                63,
                56,
            ),
            false => (
                CastleRigthsMask::BlackKingside,
                CastleRigthsMask::BlackQueenside,
                7,
                0,
            ),
        };
        match piece {
            Ok(Piece::WhiteRook | Piece::BlackRook) => {
                if mov.from.as_ix() == ks_rook {
                    self.castling.unset(ks_right)
                }
                if mov.from.as_ix() == qs_rook {
                    self.castling.unset(qs_right);
                }
            }
            Ok(Piece::WhiteKing | Piece::BlackKing) => {
                self.castling.unset(ks_right);
                self.castling.unset(qs_right);
            }
            _ => {}
        }
    }

    fn move_piece(&mut self, mov: Move) {
        let final_piece = match mov.promotion {
            Some(p) => p.into(),
            None => self.board[mov.from.as_ix() as usize],
        };
        self.board[mov.to.as_ix() as usize] = final_piece;
        self.board[mov.from.as_ix() as usize] = 0;
        if let Some(captured) = mov.en_passant {
            self.board[captured.as_ix() as usize] = 0;
        }
        self.last_move = Some(mov);
    }

    pub fn grab_piece(&mut self, ix: u8) -> Result<()> {
        if !self.in_bounds(ix) {
            return Err(BoardError::OutOfBounds.into());
        }
        let piece = Piece::try_from(self.board[ix as usize])?;
        if piece.is_white() != self.white_to_move {
            return Err(MoveError::WrongTurn.into());
        }
        self.grabbed_piece = Some(ix);
        Ok(())
    }

    pub fn drop_piece(&mut self, ix: u8) -> Result<()> {
        match self.grabbed_piece {
            Some(grabbed) => {
                let piece = Piece::try_from(self.board[grabbed as usize])?;
                let promotion = match piece {
                    Piece::BlackPawn if ix > 55 => Some(Piece::BlackQueen),
                    Piece::WhitePawn if ix < 8 => Some(Piece::WhiteQueen),
                    _ => None,
                };
                let castling = match piece {
                    Piece::BlackKing if grabbed == 4 && ix == 6 => Some((7, 5)),
                    Piece::BlackKing if grabbed == 4 && ix == 2 => Some((0, 3)),
                    Piece::WhiteKing if grabbed == 60 && ix == 62 => Some((63, 61)),
                    Piece::WhiteKing if grabbed == 60 && ix == 58 => Some((56, 59)),
                    _ => None,
                };
                let castling = castling
                    .map(|(from, to)| (Position::Index { ix: from }, Position::Index { ix: to }));
                self.make_move(Move::new_with_all(
                    Position::Index { ix: grabbed },
                    Position::Index { ix },
                    promotion,
                    None,
                    castling,
                ))?;
                self.grabbed_piece = None;
                Ok(())
            }
            None => Err(MoveError::NoPieceGrabbed.into()),
        }
    }

    pub fn has_grabbed_piece(&self) -> bool {
        self.grabbed_piece.is_some()
    }

    pub fn is_legal(&self, mov: &Move) -> bool {
        self.get_legal_moves().contains(mov)
    }

    pub fn get_all_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        for i in 0..64 {
            let piece = match Piece::try_from(self.board[i]) {
                Ok(p) => p,
                Err(_e) => continue,
            };
            if piece.is_white() != self.white_to_move {
                continue;
            }
            let mut piece_moves = piece.get_moves(
                &self.board,
                i as u8,
                self.last_move,
                self.castling,
                &self.threatmap,
            );
            moves.append(&mut piece_moves);
        }
        moves
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        let mut copy = self.clone();
        let moves = copy.get_all_moves();
        moves
            .into_iter()
            .filter(|mov| !copy.leaves_king_in_check(*mov))
            .collect()
    }

    pub fn leaves_king_in_check(&mut self, mov: Move) -> bool {
        let backup_from = self.board[mov.from.as_ix() as usize];
        let backup_to = self.board[mov.to.as_ix() as usize];
        self.move_piece(mov);
        self.update_threatmap(); // in case of discovered checks
        let king_code = match self.white_to_move {
            true => Piece::WhiteKing.into(),
            false => Piece::BlackKing.into(),
        };
        let king_ix: usize = self.board.iter().position(|&p| p == king_code).unwrap();
        let check = self.threatmap[king_ix] > 0;
        self.board[mov.from.as_ix() as usize] = backup_from;
        self.board[mov.to.as_ix() as usize] = backup_to;
        self.update_threatmap();
        check
    }

    pub fn pass_turn(&mut self) {
        self.white_to_move = !self.white_to_move;
        self.update_threatmap();
    }
}

#[derive(Clone, Debug)]
pub struct Board {
    state: BoardState,
    flipped_board: bool,
}

impl Board {
    pub fn from_fen(fen: String) -> Result<Board> {
        Ok(Board {
            state: BoardState::from_fen(fen)?,
            flipped_board: false,
        })
    }

    pub fn as_fen(&self) -> String {
        self.state.as_fen()
    }

    pub fn in_bounds(&self, pos: Position) -> bool {
        self.state.in_bounds(pos.as_ix())
    }

    pub fn make_move(&mut self, mov: Move) -> Result<()> {
        self.state.make_move(mov)
    }

    pub fn grab_piece(&mut self, pos: Position) -> Result<()> {
        self.state.grab_piece(pos.as_ix())
    }

    pub fn drop_piece(&mut self, pos: Position) -> Result<()> {
        self.state.drop_piece(pos.as_ix())
    }

    pub fn has_grabbed_piece(&self) -> bool {
        self.state.has_grabbed_piece()
    }

    pub fn white_to_move(&self) -> bool {
        self.state.white_to_move
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        self.state.get_legal_moves()
    }

    pub fn pass_turn(&mut self) {
        self.state.pass_turn()
    }

    pub fn set_flipped(&mut self, flipped: bool) {
        self.flipped_board = flipped;
    }

    pub fn get_history(&self) -> Vec<Move> {
        self.state.history.clone()
    }
}

impl Widget for Board {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        if area.area() == 0 {
            return;
        }
        let highlight_squares = self.get_grabbed_piece_highlights();
        let mut rows = Vec::with_capacity(8);
        for mut r in 0..8 {
            if self.flipped_board {
                r = 7 - r;
            }
            let mut row = Vec::with_capacity(8);
            for c in 0..8 {
                let ix: u8 = r * 8 + c;
                let style = self.get_square_style(c, r, &highlight_squares);
                let text = self.get_piece_text(ix);
                let cell = Cell::from(text).style(style);
                row.push(cell);
            }
            rows.push(Row::new(row).height(2));
        }
        Table::new(rows)
            .style(Style::default().fg(Color::White))
            .column_spacing(0)
            .widths(&[
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(4),
            ])
            .block(Block::default().title("Board").borders(Borders::ALL))
            .render(area, buf);
    }
}

/// Implementation of render helper methods
impl Board {
    fn get_piece_text(&self, ix: u8) -> String {
        let piece = Piece::try_from(self.state.board[ix as usize]);
        let char = match piece {
            Ok(p) => p.to_string(),
            Err(PieceError::NoPieceFound) => String::new(),
            Err(_e) => String::from("?"), // TODO: Should log issue to console
        };
        match piece {
            Ok(p) if p.is_white() => format!(" {}", char),
            Ok(p) if !p.is_white() => format!("\n {}", char),
            _ => char,
        }
    }

    fn get_square_style(&self, col: u8, row: u8, highlights: &[Option<Color>]) -> Style {
        let ix = row * 8 + col;
        if let Some(Some(c)) = highlights.get(ix as usize) {
            return Style::default().bg(*c);
        }
        match col + row {
            _ if self.state.grabbed_piece == Some(ix) => Style::default().bg(Color::LightRed),
            x if x % 2 != 0 => Style::default().bg(Color::DarkGray),
            x if x % 2 == 0 => Style::default().bg(Color::Gray),
            _ => panic!("invalid remainder"),
        }
    }

    fn get_grabbed_piece_highlights(&self) -> Vec<Option<Color>> {
        let mut highlights = vec![None; 64];
        match self.state.grabbed_piece {
            Some(ix) => {
                let piece = match Piece::try_from(self.state.board[ix as usize]) {
                    Ok(p) => p,
                    Err(_e) => return highlights,
                };
                let mut copy = self.state.clone();
                piece
                    .get_moves(
                        &self.state.board,
                        ix,
                        self.state.last_move,
                        self.state.castling,
                        &self.state.threatmap,
                    )
                    .into_iter()
                    .filter(|m| !copy.leaves_king_in_check(*m))
                    .map(|m| m.to)
                    .for_each(|p| highlights[p.as_ix() as usize] = Some(Color::LightGreen));
                highlights
            }
            None => highlights,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Position {
    /// Algebraic Positions treat the board as a standard chess board with ranks and files.
    /// Both rank and file are 0-based integers, so a1 is (0, 0) and 8h is (7, 7)
    Algebraic { rank: u8, file: u8 },

    /// Relative Positions treat the board as a 2d array. This is useful for translating
    /// screen positions to board positions.
    /// For example, the following comparisons are true:
    ///     * a8: Position::Algebraic {rank: 0, file: 7} == Position::Relative { col: 0, row: 0 }
    ///     * h1: Position::Algebraic {rank: 7, file: 0} == Position::Relative { col: 7, row: 7 }
    Relative { col: u8, row: u8, flip: bool },

    /// Index Positions are just the index of the square in the 1d board array.
    Index { ix: u8 },
}

impl Position {
    pub fn as_ix(&self) -> u8 {
        match self {
            Position::Algebraic { rank, file } => move_to_ix(*rank, *file),
            Position::Relative { col, row, flip } => match *flip {
                true => move_to_ix(*col, *row),
                false => col + row * 8,
            },
            Position::Index { ix } => *ix,
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ix = self.as_ix();
        let rank = ix / 8;
        let file = ix % 8;
        write!(f, "{}{}", (file + 97) as char, (8 - rank))
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.as_ix() == other.as_ix()
    }
}

type AuxMove = (Position, Position);

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub from: Position,
    pub to: Position,
    pub promotion: Option<Piece>,
    pub en_passant: Option<Position>,
    pub castling: Option<AuxMove>,
    pub piece: Option<Piece>,
}

impl Move {
    pub fn new_with_all(
        from: Position,
        to: Position,
        promotion: Option<Piece>,
        en_passant: Option<Position>,
        castling: Option<AuxMove>,
    ) -> Move {
        Move {
            from,
            to,
            promotion,
            en_passant,
            castling,
            piece: None,
        }
    }
    pub fn new(from: Position, to: Position) -> Move {
        Move::new_with_all(from, to, None, None, None)
    }

    pub fn new_promotion(from: Position, to: Position, promotion: Option<Piece>) -> Move {
        Move::new_with_all(from, to, promotion, None, None)
    }

    pub fn new_enpassant(from: Position, to: Position, en_passant: Position) -> Move {
        Move::new_with_all(from, to, None, Some(en_passant), None)
    }

    pub fn new_castling(from: Position, to: Position, castling: AuxMove) -> Move {
        Move::new_with_all(from, to, None, None, Some(castling))
    }

    pub fn castle_long(white_to_move: bool) -> Move {
        let file = Move::get_king_file(white_to_move);
        let king = Position::Algebraic { rank: 4, file };
        let rook = Position::Algebraic { rank: 0, file };
        let king_to = Position::Algebraic { rank: 2, file };
        let rook_to = Position::Algebraic { rank: 3, file };
        Move::new_castling(king, king_to, (rook, rook_to))
    }

    pub fn castle_short(white_to_move: bool) -> Move {
        let file = Move::get_king_file(white_to_move);
        let king = Position::Algebraic { rank: 4, file };
        let rook = Position::Algebraic { rank: 7, file };
        let king_to = Position::Algebraic { rank: 6, file };
        let rook_to = Position::Algebraic { rank: 5, file };
        Move::new_castling(king, king_to, (rook, rook_to))
    }

    fn get_king_file(white_to_move: bool) -> u8 {
        match white_to_move {
            true => 4,
            false => 7,
        }
    }

    pub fn set_piece(&mut self, piece: Piece) {
        self.piece = Some(piece);
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.promotion == other.promotion
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut mov = format!("{}{}", self.from, self.to);
        if let Some(p) = self.promotion {
            mov.push_str(&p.to_string());
        }
        write!(f, "{}", mov)
    }
}

fn move_to_ix(c: u8, r: u8) -> u8 {
    // there surely is a better way to do this but can't think of it now
    let m = vec![
        vec![56, 57, 58, 59, 60, 61, 62, 63],
        vec![48, 49, 50, 51, 52, 53, 54, 55],
        vec![40, 41, 42, 43, 44, 45, 46, 47],
        vec![32, 33, 34, 35, 36, 37, 38, 39],
        vec![24, 25, 26, 27, 28, 29, 30, 31],
        vec![16, 17, 18, 19, 20, 21, 22, 23],
        vec![8, 9, 10, 11, 12, 13, 14, 15],
        vec![0, 1, 2, 3, 4, 5, 6, 7],
    ];
    m[r as usize][c as usize]
}

#[cfg(test)]
mod test {
    use crate::board::{move_to_ix, Position};

    #[test]
    fn test_algebraic() {
        for c in 0..8 {
            for r in 0..8 {
                let pos = Position::Algebraic { rank: c, file: r };
                assert_eq!(pos.as_ix(), move_to_ix(c, r));
            }
        }
    }

    #[test]
    fn test_comparison() {
        let comps = [
            // a8
            (
                Position::Algebraic { rank: 0, file: 7 },
                Position::Relative {
                    col: 0,
                    row: 0,
                    flip: false,
                },
            ),
            // h1
            (
                Position::Algebraic { rank: 7, file: 0 },
                Position::Relative {
                    col: 7,
                    row: 7,
                    flip: false,
                },
            ),
        ];
        for (a, b) in comps.iter() {
            assert_eq!(a, b, "left: {} != right: {}", a.as_ix(), b.as_ix());
        }
    }
}
