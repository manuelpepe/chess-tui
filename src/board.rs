use std::fmt::Display;

use tui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};

#[derive(Clone, Debug)]
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
}

impl From<u8> for Piece {
    fn from(value: u8) -> Self {
        match value {
            0b00000001 => Piece::WhiteKing,
            0b00000010 => Piece::WhiteQueen,
            0b00000100 => Piece::WhiteRook,
            0b00001000 => Piece::WhiteBishop,
            0b00010000 => Piece::WhiteKnight,
            0b00100000 => Piece::WhitePawn,
            0b01000001 => Piece::BlackKing,
            0b01000010 => Piece::BlackQueen,
            0b01000100 => Piece::BlackRook,
            0b01001000 => Piece::BlackBishop,
            0b01010000 => Piece::BlackKnight,
            0b01100000 => Piece::BlackPawn,
            _ => panic!("unkown piece encoding"),
        }
    }
}

impl Into<u8> for Piece {
    fn into(self) -> u8 {
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

pub struct Board {}

impl Board {
    pub fn new() -> Board {
        Board {}
    }
}

impl Widget for Board {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        if area.area() == 0 {
            return;
        }

        let board_data = vec![
            vec![
                Piece::BlackRook,
                Piece::BlackKnight,
                Piece::BlackBishop,
                Piece::BlackQueen,
                Piece::BlackKing,
                Piece::BlackBishop,
                Piece::BlackKnight,
                Piece::BlackRook,
            ],
            vec![
                Piece::BlackPawn,
                Piece::BlackPawn,
                Piece::BlackPawn,
                Piece::BlackPawn,
                Piece::BlackPawn,
                Piece::BlackPawn,
                Piece::BlackPawn,
                Piece::BlackPawn,
            ],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![
                Piece::WhitePawn,
                Piece::WhitePawn,
                Piece::WhitePawn,
                Piece::WhitePawn,
                Piece::WhitePawn,
                Piece::WhitePawn,
                Piece::WhitePawn,
                Piece::WhitePawn,
            ],
            vec![
                Piece::WhiteRook,
                Piece::WhiteKnight,
                Piece::WhiteBishop,
                Piece::WhiteQueen,
                Piece::WhiteKing,
                Piece::WhiteBishop,
                Piece::WhiteKnight,
                Piece::WhiteRook,
            ],
        ];

        let mut rows = Vec::with_capacity(8);
        for i in 0..8 {
            let mut row = Vec::with_capacity(8);
            for j in 0..8 {
                let style = match i + j {
                    i if i % 2 != 0 => Style::default().bg(Color::DarkGray),
                    i if i % 2 == 0 => Style::default().bg(Color::White),
                    _ => panic!("invalid remainder"),
                };
                let piece = match board_data[i].get(j) {
                    Some(p) => p.to_string(),
                    None => String::new(),
                };
                let cell = Cell::from(format!("{}", piece)).style(style);
                row.push(cell);
            }
            rows.push(Row::new(row).bottom_margin(1));
        }

        let _table = Table::new(rows)
            .style(Style::default().fg(Color::White))
            .widths(&[
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .block(Block::default().borders(Borders::ALL))
            .render(area, buf);
    }
}
