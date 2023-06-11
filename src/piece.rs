use std::fmt::Display;
use thiserror::Error;

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
        return self.as_u8() < 64;
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
}

impl TryFrom<u8> for Piece {
    type Error = PieceError;

    fn try_from(value: u8) -> Result<Self, PieceError> {
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
            0b00000000 => Err(PieceError::NoPieceFound),
            _ => Err(PieceError::PieceEncodingError),
        }
    }
}

impl TryFrom<char> for Piece {
    type Error = PieceError;

    fn try_from(value: char) -> Result<Self, PieceError> {
        match value {
            'k' => Ok(Piece::BlackKing),
            'q' => Ok(Piece::BlackQueen),
            'r' => Ok(Piece::BlackRook),
            'b' => Ok(Piece::BlackBishop),
            'n' => Ok(Piece::BlackKnight),
            'p' => Ok(Piece::BlackPawn),
            'K' => Ok(Piece::WhiteKing),
            'Q' => Ok(Piece::WhiteQueen),
            'R' => Ok(Piece::WhiteRook),
            'B' => Ok(Piece::WhiteBishop),
            'N' => Ok(Piece::WhiteKnight),
            'P' => Ok(Piece::WhitePawn),
            _ => Err(PieceError::UnkownFENCharacter),
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
pub enum PieceError {
    #[error("piece encoding error")]
    PieceEncodingError,

    #[error("no piece at the given position")]
    NoPieceFound,

    #[error("unkown FEN character")]
    UnkownFENCharacter,
}
