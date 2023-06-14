use anyhow::Result;
use thiserror::Error;
use tui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};

use crate::{
    fen::FEN,
    piece::{Piece, PieceError},
};

#[derive(Clone, Copy, Error, Debug)]
pub enum ParsingError {
    #[error("error parsing fen")]
    ErrorParsingFEN,

    #[error("error parsing move")]
    MoveParsingError,
}

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

#[derive(Clone, Copy, Debug)]
pub struct BoardState {
    pub board: [u8; 64],
    pub white_to_move: bool,
    pub grabbed_piece: Option<u8>,
    pub last_move: Option<Move>,

    /// Castling rights, 2 bits for each side, 4 bit padding:
    /// [XXXX KQkq]
    pub castling: u8,
}

impl BoardState {
    pub fn from_fen(value: String) -> Result<Self> {
        let fen = FEN::parse(value)?;
        let mut state = BoardState {
            board: fen.board,
            white_to_move: fen.white_to_move,
            grabbed_piece: None,
            last_move: None,
            castling: fen.castling,
        };
        Ok(state)
    }

    pub fn as_fen(&self) -> String {
        FEN {
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
            return Ok(());
        }
        if !self.is_legal(&mov) {
            return Err(MoveError::IllegalMove { mov: mov }.into());
        };
        self.move_piece(mov);
        self.white_to_move = !self.white_to_move;
        Ok(())
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
                let promotion = match Piece::try_from(self.board[grabbed as usize]) {
                    Ok(Piece::BlackPawn) if ix > 55 => Some(Piece::BlackQueen),
                    Ok(Piece::WhitePawn) if ix < 8 => Some(Piece::WhiteQueen),
                    _ => None,
                };
                self.make_move(Move::new_promotion(
                    Position::Index { ix: grabbed },
                    Position::Index { ix },
                    promotion,
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
            let mut piece_moves = piece.get_moves(&self.board, i as u8, self.last_move);
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
        // backup for later restore
        let backup_from = self.board[mov.from.as_ix() as usize];
        let backup_to = self.board[mov.to.as_ix() as usize];
        // change state
        self.move_piece(mov);
        self.white_to_move = !self.white_to_move;
        // look for checks
        let king_code = match self.white_to_move {
            true => Piece::BlackKing.into(),
            false => Piece::WhiteKing.into(),
        };
        let king_ix = self.board.iter().position(|&p| p == king_code).unwrap();
        let check = self
            .get_all_moves()
            .iter()
            .any(|m| m.to.as_ix() as usize == king_ix);
        // restore state
        self.board[mov.from.as_ix() as usize] = backup_from;
        self.board[mov.to.as_ix() as usize] = backup_to;
        self.white_to_move = !self.white_to_move;

        check
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Board {
    state: BoardState,
}

impl Board {
    pub fn new() -> Board {
        Board {
            state: BoardState {
                board: [0; 64],
                white_to_move: true,
                grabbed_piece: None,
                last_move: None,
                castling: 0,
            },
        }
    }

    pub fn from_fen(fen: String) -> Result<Board> {
        Ok(Board {
            state: BoardState::from_fen(fen)?,
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
}

impl Widget for Board {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        if area.area() == 0 {
            return;
        }
        let highlight_squares = self.get_grabbed_piece_highlights();
        let mut rows = Vec::with_capacity(8);
        for r in 0..8 {
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

        let _table = Table::new(rows)
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

    fn get_square_style(&self, col: u8, row: u8, highlights: &Vec<Option<Color>>) -> Style {
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
                    .get_moves(&self.state.board, ix, self.state.last_move)
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
    Relative { col: u8, row: u8 },

    /// Index Positions are just the index of the square in the 1d board array.
    Index { ix: u8 },
}

impl Position {
    pub fn as_ix(&self) -> u8 {
        match self {
            Position::Algebraic { rank, file } => move_to_ix(*rank, *file),
            Position::Relative { col, row } => col + row * 8,
            Position::Index { ix } => *ix,
        }
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.as_ix() == other.as_ix()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub from: Position,
    pub to: Position,
    pub promotion: Option<Piece>,
    pub en_passant: Option<Position>,
}

impl Move {
    pub fn new(from: Position, to: Position) -> Move {
        Move {
            from: from,
            to: to,
            promotion: None,
            en_passant: None,
        }
    }

    pub fn new_promotion(from: Position, to: Position, promotion: Option<Piece>) -> Move {
        Move {
            from: from,
            to: to,
            promotion: promotion,
            en_passant: None,
        }
    }

    pub fn new_enpassant(from: Position, to: Position, en_passant: Position) -> Move {
        Move {
            from: from,
            to: to,
            promotion: None,
            en_passant: Some(en_passant),
        }
    }

    pub fn in_bounds(&self, board: Board) -> bool {
        board.in_bounds(self.from) && board.in_bounds(self.to)
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.promotion == other.promotion
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
        vec![08, 09, 10, 11, 12, 13, 14, 15],
        vec![00, 01, 02, 03, 04, 05, 06, 07],
    ];
    return m[r as usize][c as usize];
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
                Position::Relative { col: 0, row: 0 },
            ),
            // h1
            (
                Position::Algebraic { rank: 7, file: 0 },
                Position::Relative { col: 7, row: 7 },
            ),
        ];
        for (a, b) in comps.iter() {
            assert_eq!(a, b, "left: {} != right: {}", a.as_ix(), b.as_ix());
        }
    }
}
