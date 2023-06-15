use std::fmt::{Display, Write};
use thiserror::Error;

use crate::board::{Move, Position};

#[derive(Clone, Copy, Debug)]
pub enum CastleRigthsMask {
    WhiteKingside = 8,
    WhiteQueenside = 4,
    BlackKingside = 2,
    BlackQueenside = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CastleRights {
    rights: u8,
}

impl CastleRights {
    pub fn set(&mut self, mask: CastleRigthsMask) {
        self.rights |= mask as u8;
    }

    pub fn unset(&mut self, mask: CastleRigthsMask) {
        self.rights &= !(mask as u8);
    }

    pub fn get(&self, mask: CastleRigthsMask) -> bool {
        self.rights & (mask as u8) > 0
    }

    pub fn get_u8(&self) -> u8 {
        self.rights
    }
}

impl Display for CastleRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.get(CastleRigthsMask::WhiteKingside) {
            f.write_char('K')?;
        }
        if self.get(CastleRigthsMask::WhiteQueenside) {
            f.write_char('Q')?;
        }
        if self.get(CastleRigthsMask::BlackKingside) {
            f.write_char('k')?;
        }
        if self.get(CastleRigthsMask::BlackQueenside) {
            f.write_char('q')?;
        }
        Ok(())
    }
}

impl From<&str> for CastleRights {
    fn from(val: &str) -> Self {
        let rights = val.chars().fold(CastleRights::default(), |mut acc, c| {
            match c {
                'K' => acc.set(CastleRigthsMask::WhiteKingside),
                'Q' => acc.set(CastleRigthsMask::WhiteQueenside),
                'k' => acc.set(CastleRigthsMask::BlackKingside),
                'q' => acc.set(CastleRigthsMask::BlackQueenside),
                _ => {}
            }
            acc
        });
        rights
    }
}

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
        v < 64
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
        std::char::from_u32(self.as_unicode()).unwrap_or('�')
    }

    pub fn get_moves(
        &self,
        board: &[u8; 64],
        position: u8,
        last_move: Option<Move>,
        castle_rights: CastleRights,
        threatmap: &[u8; 64],
    ) -> Vec<Move> {
        match *self {
            Piece::BlackKing | Piece::WhiteKing => {
                let mut moves = self.get_sliding_moves(board, position);
                moves.append(&mut self.get_castling_moves(
                    board,
                    position,
                    castle_rights,
                    threatmap,
                ));
                moves
            }
            Piece::BlackQueen | Piece::WhiteQueen => self.get_sliding_moves(board, position),
            Piece::BlackRook | Piece::WhiteRook => self.get_sliding_moves(board, position),
            Piece::BlackBishop | Piece::WhiteBishop => self.get_sliding_moves(board, position),
            Piece::BlackKnight | Piece::WhiteKnight => self.get_knight_moves(board, position),
            Piece::BlackPawn | Piece::WhitePawn => self.get_pawn_moves(board, position, last_move),
        }
    }

    fn get_castling_moves(
        &self,
        board: &[u8; 64],
        position: u8,
        castle_rights: CastleRights,
        threatmap: &[u8; 64],
    ) -> Vec<Move> {
        let king_ix = position;
        let mut moves = Vec::new();
        let (ks_right, qs_right, ks_rook, qs_rook) = match self.is_white() {
            true => (
                castle_rights.get(CastleRigthsMask::WhiteKingside),
                castle_rights.get(CastleRigthsMask::WhiteQueenside),
                63,
                56,
            ),
            false => (
                castle_rights.get(CastleRigthsMask::BlackKingside),
                castle_rights.get(CastleRigthsMask::BlackQueenside),
                7,
                0,
            ),
        };
        if ks_right {
            let rook_path_clear = path_clear(board, ks_rook, king_ix + 1, &[0; 64]);
            let king_path_clear = path_clear(board, king_ix, king_ix + 2, threatmap);
            if rook_path_clear && king_path_clear {
                moves.push(Move::new_castling(
                    Position::Index { ix: king_ix },
                    Position::Index { ix: king_ix + 2 },
                    (
                        Position::Index { ix: ks_rook },
                        Position::Index { ix: king_ix + 1 },
                    ),
                ));
            }
        }
        if qs_right {
            let rook_path_clear = path_clear(board, qs_rook, king_ix - 1, &[0; 64]);
            let king_path_clear = path_clear(board, king_ix, king_ix - 2, threatmap);
            if rook_path_clear && king_path_clear {
                moves.push(Move::new_castling(
                    Position::Index { ix: king_ix },
                    Position::Index { ix: king_ix - 2 },
                    (
                        Position::Index { ix: qs_rook },
                        Position::Index { ix: king_ix - 1 },
                    ),
                ));
            }
        }
        moves
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
                if !(0..=63).contains(&pos) {
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
                        ));
                    }
                    break;
                }
                // add move to valid empty square
                moves.push(Move::new(
                    Position::Index { ix: position },
                    Position::Index { ix: pos as u8 },
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
            if !(0..=63).contains(&new_pos) {
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
                    ));
                }
                continue;
            }
            // add move to valid empty square
            moves.push(Move::new(
                Position::Index { ix: position },
                Position::Index { ix: new_pos as u8 },
            ));
        }
        moves
    }

    fn get_pawn_moves(&self, board: &[u8; 64], position: u8, last_move: Option<Move>) -> Vec<Move> {
        let mut moves = Vec::new();
        let direction: i8 = if self.is_white() { -1 } else { 1 };
        let is_first_move = (!self.is_white() && position < 16 && position > 7)
            || (self.is_white() && position < 56 && position > 47);
        // forwards move
        let ahead_pos = position as i8 + 8 * direction;
        if (0..64).contains(&ahead_pos) && board[ahead_pos as usize] == 0 {
            self.add_with_promotions(&mut moves, position, ahead_pos as u8);
            if is_first_move {
                let two_ahead_pos = position as i8 + 16 * direction;
                if board[two_ahead_pos as usize] == 0 {
                    self.add_with_promotions(&mut moves, position, two_ahead_pos as u8);
                }
            }
        }
        // left capture
        let left_pos = position as i8 + 8 * direction - 1;
        if (0..64).contains(&left_pos) && left_pos % 8 != 7 {
            if let Ok(p) = Piece::try_from(board[left_pos as usize]) {
                if p.is_white() != self.is_white() {
                    self.add_with_promotions(&mut moves, position, left_pos as u8);
                }
            }
        }
        // right capture
        let right_pos = position as i8 + 8 * direction + 1;
        if (0..64).contains(&right_pos) && right_pos % 8 != 0 {
            if let Ok(p) = Piece::try_from(board[right_pos as usize]) {
                if p.is_white() != self.is_white() {
                    self.add_with_promotions(&mut moves, position, right_pos as u8);
                }
            }
        }

        // en passant
        if let Some(m) = self.get_en_passant(board, position, last_move) {
            moves.push(m);
        }

        moves
    }

    fn get_en_passant(
        &self,
        board: &[u8; 64],
        position: u8,
        last_move: Option<Move>,
    ) -> Option<Move> {
        // check there is a last move
        let last_move = match last_move {
            Some(m) => m,
            None => return None,
        };
        // check last move was a pawn
        match Piece::try_from(board[last_move.to.as_ix() as usize]) {
            Ok(p) => {
                if p != Piece::WhitePawn && p != Piece::BlackPawn {
                    return None;
                }
            }
            Err(_) => return None,
        };
        // usefull vars
        let last_from_file = last_move.from.as_ix() / 8;
        let last_to_rank = last_move.to.as_ix() % 8;
        let last_to_file = last_move.to.as_ix() / 8;
        let self_rank = position % 8_u8;
        let self_file = position / 8_u8;
        // check double pawn push
        if last_from_file.abs_diff(last_to_file) != 2 {
            return None; // skip if last move was not a double pawn move
        }
        // check side by side
        if last_to_file != self_file {
            return None;
        }
        // check 1 rank offset
        if last_to_rank.abs_diff(self_rank) != 1 {
            return None;
        }
        let direction: i8 = if self.is_white() { -1 } else { 1 };
        let capture_square = position + last_to_rank - self_rank;
        let dest_square = (capture_square as i8 + 8 * direction) as u8;
        Some(Move::new_enpassant(
            Position::Index { ix: position },
            Position::Index { ix: dest_square },
            Position::Index { ix: capture_square },
        ))
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
            moves.push(Move::new_promotion(
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

impl From<Piece> for u8 {
    fn from(val: Piece) -> Self {
        match val {
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

impl From<Piece> for char {
    fn from(val: Piece) -> Self {
        match val {
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

fn path_clear(board: &[u8; 64], from: u8, to: u8, threatmap: &[u8; 64]) -> bool {
    for i in from + 1..=to {
        if board[i as usize] != 0 {
            return false;
        }
        if threatmap[i as usize] != 0 {
            return false;
        }
    }
    true
}
