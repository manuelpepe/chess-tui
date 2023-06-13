use std::fmt::Display;
use thiserror::Error;

use crate::board::{Move, Position};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        let v: u8 = (*self).into();
        return v < 64;
    }

    pub fn white_pieces() -> Vec<Piece> {
        vec![
            Piece::WhiteKing,
            Piece::WhiteQueen,
            Piece::WhiteRook,
            Piece::WhiteBishop,
            Piece::WhiteKnight,
            Piece::WhitePawn,
        ]
    }

    pub fn black_pieces() -> Vec<Piece> {
        vec![
            Piece::BlackKing,
            Piece::BlackQueen,
            Piece::BlackRook,
            Piece::BlackBishop,
            Piece::BlackKnight,
            Piece::BlackPawn,
        ]
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

    fn as_unicode_char(self) -> char {
        match std::char::from_u32(self.as_unicode()) {
            Some(c) => c,
            None => '�',
        }
    }

    pub fn get_moves(&self, board: &[u8; 64], position: u8) -> Vec<Move> {
        match *self {
            Piece::BlackKing | Piece::WhiteKing => self.get_sliding_moves(board, position),
            Piece::BlackQueen | Piece::WhiteQueen => self.get_sliding_moves(board, position),
            Piece::BlackRook | Piece::WhiteRook => self.get_sliding_moves(board, position),
            Piece::BlackBishop | Piece::WhiteBishop => self.get_sliding_moves(board, position),
            Piece::BlackKnight | Piece::WhiteKnight => self.get_knight_moves(board, position),
            Piece::BlackPawn | Piece::WhitePawn => self.get_pawn_moves(board, position),
        }
    }

    fn get_sliding_moves(&self, board: &[u8; 64], position: u8) -> Vec<Move> {
        let mut moves = Vec::new();
        let piece = match Piece::try_from(board[position as usize]) {
            Ok(p) => p,
            Err(_) => return moves,
        };
        let directions = vec![8, -8, 1, -1, 7, -7, 9, -9];
        let directions = match piece {
            Piece::WhiteQueen | Piece::BlackQueen => directions,
            Piece::WhiteKing | Piece::BlackKing => directions,
            Piece::WhiteRook | Piece::BlackRook => directions[0..4].to_vec(),
            Piece::WhiteBishop | Piece::BlackBishop => directions[4..8].to_vec(),
            _ => return moves,
        };
        for direction in directions {
            let mut pos = position as i8;
            loop {
                let last_rank = pos % 8;
                pos += direction;
                let new_rank = pos % 8;

                // check bounds
                if pos < 0 || pos > 63 {
                    break;
                }
                // check if position has wrapped to the left
                if last_rank == 0 && new_rank == 7 {
                    break;
                }
                // check if position has wrapped to the right
                if last_rank == 7 && new_rank == 0 {
                    break;
                }
                // add move captures
                if let Ok(p) = Piece::try_from(board[pos as usize]) {
                    if p.is_white() != self.is_white() {
                        moves.push(Move::new(
                            Position::Index { ix: position },
                            Position::Index { ix: pos as u8 },
                            None,
                        ));
                    }
                    break;
                }
                // add move to valid empty square
                moves.push(Move::new(
                    Position::Index { ix: position },
                    Position::Index { ix: pos as u8 },
                    None,
                ));
                // only go 1 depth each direction for king
                if piece == Piece::WhiteKing || piece == Piece::BlackKing {
                    break;
                }
            }
        }
        moves
    }

    fn get_knight_moves(&self, board: &[u8; 64], position: u8) -> Vec<Move> {
        let mut moves = Vec::new();
        let directions = vec![6, -6, 10, -10, 15, -15, 17, -17];
        for direction in directions {
            let new_pos = position as i8 + direction;
            let cur_rank = position % 8;
            let new_rank = new_pos % 8;
            // check bounds
            if new_pos < 0 || new_pos > 63 {
                continue;
            }
            // check if position has wrapped to the left
            if (cur_rank == 0 || cur_rank == 1) && (new_rank == 7 || new_rank == 6) {
                continue;
            }
            // check if position has wrapped to the right
            if (cur_rank == 6 || cur_rank == 7) && (new_rank == 0 || new_rank == 1) {
                continue;
            }
            // add move captures
            if let Ok(p) = Piece::try_from(board[new_pos as usize]) {
                if p.is_white() != self.is_white() {
                    moves.push(Move::new(
                        Position::Index { ix: position },
                        Position::Index { ix: new_pos as u8 },
                        None,
                    ));
                }
                continue;
            }
            // add move to valid empty square
            moves.push(Move::new(
                Position::Index { ix: position },
                Position::Index { ix: new_pos as u8 },
                None,
            ));
        }
        moves
    }

    fn get_pawn_moves(&self, board: &[u8; 64], position: u8) -> Vec<Move> {
        let mut moves = Vec::new();
        let direction: i8 = if self.is_white() { -1 } else { 1 };
        let is_first_move = (!self.is_white() && position < 16 && position > 7)
            || (self.is_white() && position < 56 && position > 47);
        // forwards move
        let ahead_pos = position as i8 + 8 * direction;
        if ahead_pos < 64 && ahead_pos >= 0 {
            if board[ahead_pos as usize] == 0 {
                self.add_with_promotions(&mut moves, position, ahead_pos as u8);
                if is_first_move {
                    let two_ahead_pos = position as i8 + 16 * direction;
                    if board[two_ahead_pos as usize] == 0 {
                        self.add_with_promotions(&mut moves, position, two_ahead_pos as u8);
                    }
                }
            }
        }
        // left capture
        let left_pos = position as i8 + 8 * direction - 1;
        if left_pos < 64 && left_pos >= 0 && left_pos % 8 != 7 {
            if let Ok(p) = Piece::try_from(board[left_pos as usize]) {
                if p.is_white() != self.is_white() {
                    self.add_with_promotions(&mut moves, position, left_pos as u8);
                }
            }
        }
        // right capture
        let right_pos = position as i8 + 8 * direction + 1;
        if right_pos < 64 && right_pos >= 0 && right_pos % 8 != 0 {
            if let Ok(p) = Piece::try_from(board[right_pos as usize]) {
                if p.is_white() != self.is_white() {
                    self.add_with_promotions(&mut moves, position, right_pos as u8);
                }
            }
        }
        // TODO: en passant. I need to keep track of the last move and pass it to this function.
        moves
    }

    fn add_with_promotions(&self, moves: &mut Vec<Move>, from: u8, to: u8) {
        let mut promotions = vec![None];
        let is_promoting = (self.is_white() && to < 8) || (!self.is_white() && to > 55);
        if is_promoting {
            if self.is_white() {
                promotions = Piece::white_pieces().iter().map(|p| Some(*p)).collect();
            } else {
                promotions = Piece::black_pieces().iter().map(|p| Some(*p)).collect();
            };
        };
        for piece in promotions {
            moves.push(Move::new(
                Position::Index { ix: from },
                Position::Index { ix: to },
                piece,
            ));
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

impl Into<char> for Piece {
    fn into(self) -> char {
        match self {
            Piece::BlackKing => 'k',
            Piece::BlackQueen => 'q',
            Piece::BlackRook => 'r',
            Piece::BlackBishop => 'b',
            Piece::BlackKnight => 'n',
            Piece::BlackPawn => 'p',
            Piece::WhiteKing => 'K',
            Piece::WhiteQueen => 'Q',
            Piece::WhiteRook => 'R',
            Piece::WhiteBishop => 'B',
            Piece::WhiteKnight => 'N',
            Piece::WhitePawn => 'P',
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_unicode_char().to_string().as_str())
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
