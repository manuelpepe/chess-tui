use std::fmt::Display;

use anyhow::Result;
use thiserror::Error;
use tui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};

#[derive(Clone, Copy, Debug)]
pub enum Piece {
    WhiteKing,
    WhiteQueen,
    WhiteRook,
    WhiteBishop,
    WhiteKnight,
    WhitePawn,
    BlackKing,
    BlackQueen,
    BlackRook,
    BlackBishop,
    BlackKnight,
    BlackPawn,
}

impl Piece {
    pub fn is_white(&self) -> bool {
        return self.as_u8() & 0b01000000 == 0b01000000;
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Piece::WhiteKing => 0b00000001,
            Piece::WhiteQueen => 0b00000010,
            Piece::WhiteRook => 0b00000100,
            Piece::WhiteBishop => 0b00001000,
            Piece::WhiteKnight => 0b00010000,
            Piece::WhitePawn => 0b00100000,
            Piece::BlackKing => 0b01000001,
            Piece::BlackQueen => 0b01000010,
            Piece::BlackRook => 0b01000100,
            Piece::BlackBishop => 0b01001000,
            Piece::BlackKnight => 0b01010000,
            Piece::BlackPawn => 0b01100000,
        }
    }

    pub fn as_unicode(&self) -> u32 {
        match *self {
            Piece::WhiteKing => 0x2654,
            Piece::WhiteQueen => 0x2655,
            Piece::WhiteRook => 0x2656,
            Piece::WhiteBishop => 0x2657,
            Piece::WhiteKnight => 0x2658,
            Piece::WhitePawn => 0x2659,
            Piece::BlackKing => 0x265A,
            Piece::BlackQueen => 0x265B,
            Piece::BlackRook => 0x265C,
            Piece::BlackBishop => 0x265D,
            Piece::BlackKnight => 0x265E,
            Piece::BlackPawn => 0x265F,
        }
    }

    pub fn from_fen(ch: char) -> Option<Piece> {
        match ch {
            'k' => Some(Piece::BlackKing),
            'q' => Some(Piece::BlackQueen),
            'r' => Some(Piece::BlackRook),
            'b' => Some(Piece::BlackBishop),
            'n' => Some(Piece::BlackKnight),
            'p' => Some(Piece::BlackPawn),
            'K' => Some(Piece::WhiteKing),
            'Q' => Some(Piece::WhiteQueen),
            'R' => Some(Piece::WhiteRook),
            'B' => Some(Piece::WhiteBishop),
            'N' => Some(Piece::WhiteKnight),
            'P' => Some(Piece::WhitePawn),
            _ => None,
        }
    }
}

impl TryFrom<u8> for Piece {
    type Error = ParsingError;

    fn try_from(value: u8) -> Result<Self, ParsingError> {
        match value {
            0b00000001 => Ok(Piece::WhiteKing),
            0b00000010 => Ok(Piece::WhiteQueen),
            0b00000100 => Ok(Piece::WhiteRook),
            0b00001000 => Ok(Piece::WhiteBishop),
            0b00010000 => Ok(Piece::WhiteKnight),
            0b00100000 => Ok(Piece::WhitePawn),
            0b01000001 => Ok(Piece::BlackKing),
            0b01000010 => Ok(Piece::BlackQueen),
            0b01000100 => Ok(Piece::BlackRook),
            0b01001000 => Ok(Piece::BlackBishop),
            0b01010000 => Ok(Piece::BlackKnight),
            0b01100000 => Ok(Piece::BlackPawn),
            0b00000000 => Err(ParsingError::NoPieceFound),
            _ => Err(ParsingError::PieceEncodingError),
        }
    }
}

impl Into<u8> for Piece {
    fn into(self) -> u8 {
        self.as_u8()
    }
}

impl Into<char> for &Piece {
    fn into(self) -> char {
        match std::char::from_u32(self.as_unicode()) {
            Some(c) => c,
            None => '�',
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch: char = self.into();
        f.write_str(format!("{}", ch).as_str())
    }
}

#[derive(Clone, Copy, Error, Debug)]
pub enum ParsingError {
    #[error("error parsing fen")]
    ErrorParsingFEN,

    #[error("piece encoding error")]
    PieceEncodingError,

    #[error("no piece at the given position")]
    NoPieceFound,

    #[error("error parsing move")]
    MoveParsingError,
}

#[derive(Clone, Copy, Debug)]
pub struct BoardState {
    pub board: [u8; 64],
    pub white_to_move: bool,
    pub grabbed_piece: Option<u8>,
}

impl BoardState {
    pub fn from_fen(value: String) -> Result<Self> {
        // TODO: Finish parsing fen, only position is parsed for now
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
            if let Some(piece) = Piece::from_fen(ch) {
                board[ix] = piece.into();
            }
            ix += 1;
        }
        Ok(BoardState {
            board: board,
            white_to_move: true,
            grabbed_piece: None,
        })
    }

    pub fn grab_piece(&mut self, ix: u8) -> Result<()> {
        if self.board[ix as usize] == 0 {
            return Err(ParsingError::NoPieceFound.into());
        }
        self.grabbed_piece = Some(ix);
        Ok(())
    }

    pub fn drop_piece(&mut self, ix: u8) {
        match self.grabbed_piece {
            Some(grabbed) => {
                self.board[ix as usize] = self.board[grabbed as usize];
                self.board[grabbed as usize] = 0;
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
            },
        }
    }

    pub fn from_fen(fen: String) -> Result<Board> {
        Ok(Board {
            state: BoardState::from_fen(fen)?,
        })
    }

    pub fn make_move(&mut self, mov: Move) -> Result<(u8, u8)> {
        self.state.board[mov.to.as_ix() as usize] = self.state.board[mov.from.as_ix() as usize];
        self.state.board[mov.from.as_ix() as usize] = 0;
        Ok((mov.from.as_ix(), mov.to.as_ix()))
    }

    pub fn grab_piece(&mut self, pos: Position) -> Result<()> {
        self.state.grab_piece(pos.as_ix())
    }

    pub fn drop_piece(&mut self, pos: Position) {
        self.state.drop_piece(pos.as_ix());
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
                    i if i % 2 != 0 => Style::default().bg(Color::DarkGray),
                    i if i % 2 == 0 => Style::default().bg(Color::Gray),
                    _ => panic!("invalid remainder"),
                };
                let piece = Piece::try_from(self.state.board[i * 8 + j]);
                let piece_img = match &piece {
                    Ok(p) => p.to_string(),
                    Err(ParsingError::NoPieceFound) => String::new(),
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
}

impl Position {
    pub fn as_ix(&self) -> u8 {
        match self {
            Position::Algebraic { rank, file } => move_to_ix(*rank, *file),
            Position::Relative { col, row } => col + row * 8,
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
    pub promotion: Option<u8>,
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
                let pos = Position::Algebraic { rank: r, file: c };
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
