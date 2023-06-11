use anyhow::Result;
use thiserror::Error;
use tui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};

use crate::piece::{Piece, PieceError};

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
}

#[derive(Clone, Copy, Debug)]
pub struct BoardState {
    pub board: [u8; 64],
    pub white_to_move: bool,
    pub grabbed_piece: Option<u8>,

    /// Castling rights, 2 bits for each side, 4 bit padding:
    /// [XXXX KQkq]
    pub castling: u8,
}

impl BoardState {
    pub fn from_fen(value: String) -> Result<Self> {
        let mut board = [0u8; 64];
        let position = value
            .split_whitespace()
            .nth(0)
            .ok_or_else(|| ParsingError::ErrorParsingFEN)?
            .chars();
        let mut ix = 0;
        for ch in position.into_iter() {
            if ch == '/' {
                continue;
            }
            if ch.is_numeric() {
                let skip = ch.to_digit(10).unwrap();
                ix += skip as usize;
                continue;
            }
            if let Ok(piece) = Piece::try_from(ch) {
                board[ix] = piece.into();
            }
            ix += 1;
        }
        let turn = value
            .split_whitespace()
            .nth(1)
            .unwrap_or_else(|| "w")
            .to_lowercase();
        let castling = value
            .split_whitespace()
            .nth(2)
            .unwrap_or_else(|| "")
            .chars()
            .fold(0, |acc, c| match c {
                'K' => acc + 8,
                'Q' => acc + 4,
                'k' => acc + 2,
                'q' => acc + 1,
                _ => acc,
            });
        let _enpassant = value.split_whitespace().nth(3).unwrap_or_else(|| "");
        // TODO: Parse timers
        Ok(BoardState {
            board: board,
            white_to_move: turn == "w",
            grabbed_piece: None,
            castling: castling,
        })
    }

    pub fn make_move(&mut self, mov: Move) -> (u8, u8) {
        if mov.from == mov.to {
            return (mov.from.as_ix(), mov.to.as_ix());
        }
        let final_piece = match mov.promotion {
            Some(p) => p.into(),
            None => self.board[mov.from.as_ix() as usize],
        };
        self.board[mov.to.as_ix() as usize] = final_piece;
        self.board[mov.from.as_ix() as usize] = 0;
        self.white_to_move = !self.white_to_move;
        (mov.from.as_ix(), mov.to.as_ix())
    }

    pub fn grab_piece(&mut self, ix: u8) -> Result<()> {
        let piece = Piece::try_from(self.board[ix as usize])?;
        if piece.is_white() != self.white_to_move {
            return Err(MoveError::WrongTurn.into());
        }
        self.grabbed_piece = Some(ix);
        Ok(())
    }

    pub fn drop_piece(&mut self, ix: u8) {
        match self.grabbed_piece {
            Some(grabbed) => {
                let promotion = match Piece::try_from(self.board[grabbed as usize]) {
                    Ok(Piece::BlackPawn) if ix > 55 => Some(Piece::BlackQueen),
                    Ok(Piece::WhitePawn) if ix < 8 => Some(Piece::WhiteQueen),
                    _ => None,
                };
                self.make_move(Move {
                    from: Position::Index { ix: grabbed },
                    to: Position::Index { ix },
                    promotion: promotion,
                });
                self.grabbed_piece = None;
            }
            None => {
                // TODO: Should log issue to console in debug
            }
        }
    }

    pub fn has_grabbed_piece(&self) -> bool {
        self.grabbed_piece.is_some()
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
                castling: 0,
            },
        }
    }

    pub fn from_fen(fen: String) -> Result<Board> {
        Ok(Board {
            state: BoardState::from_fen(fen)?,
        })
    }

    pub fn make_move(&mut self, mov: Move) -> (u8, u8) {
        self.state.make_move(mov)
    }

    pub fn grab_piece(&mut self, pos: Position) -> Result<()> {
        self.state.grab_piece(pos.as_ix())
    }

    pub fn drop_piece(&mut self, pos: Position) {
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
        let mut rows = Vec::with_capacity(8);
        for i in 0..8 {
            let mut row = Vec::with_capacity(8);
            for j in 0..8 {
                let style = match i + j {
                    _ if self.state.grabbed_piece == Some(i * 8 + j) => {
                        Style::default().bg(Color::LightRed)
                    }
                    x if x % 2 != 0 => Style::default().bg(Color::DarkGray),
                    x if x % 2 == 0 => Style::default().bg(Color::Gray),
                    _ => panic!("invalid remainder"),
                };
                let piece = Piece::try_from(self.state.board[(i * 8 + j) as usize]);
                let piece_img = match &piece {
                    Ok(p) => p.to_string(),
                    Err(PieceError::NoPieceFound) => String::new(),
                    Err(_e) => String::from("???"), // TODO: Should log issue to console
                };
                let position_format = match piece {
                    Ok(p) if p.is_white() => format!(" {}", piece_img),
                    Ok(p) if !p.is_white() => format!("\n {}", piece_img),
                    _ => piece_img,
                };
                let cell = Cell::from(position_format).style(style);
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

#[derive(Clone, Copy, Debug)]
pub enum Position {
    /// Algebraic positions dont know about the board representation, and instead refer
    /// to squares by rank and file.
    /// Both rank and file are 0-based integers, so a1 is (0, 0) and 8h is (7, 7)
    Algebraic { rank: u8, file: u8 },

    /// Relative positions know about the board representation and can represent
    /// a square from the indexes of the board as an array.
    /// For example, the following comparisons are true:
    ///     * a8: Position::Algebraic {rank: 0, file: 7} == Position::Relative { col: 0, row: 0 }
    ///     * h1: Position::Algebraic {rank: 7, file: 0} == Position::Relative { col: 7, row: 7 }
    Relative { col: u8, row: u8 },

    /// Index positions are just the index of the square in the board array.
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

pub struct Move {
    pub from: Position,
    pub to: Position,
    pub promotion: Option<Piece>,
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
