use std::fmt::{Display, Write};

use anyhow::Result;

use crate::{board::ParsingError, piece::Piece};

pub struct FEN {
    pub board: [u8; 64],
    pub white_to_move: bool,
    pub castling: u8,
}

impl FEN {
    pub fn parse(value: String) -> Result<Self> {
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
        Ok(FEN {
            board: board,
            white_to_move: turn == "w",
            castling: castling,
        })
    }
}

impl Display for FEN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in 0..8 {
            let mut empty = 0;
            for r in 0..8 {
                let piece = self.board[c * 8 + r];
                if piece == 0 {
                    empty += 1;
                } else {
                    if empty > 0 {
                        f.write_str(&empty.to_string())?;
                        empty = 0;
                    }
                    let fenpiece: char = Piece::try_from(piece).unwrap().into();
                    f.write_str(fenpiece.to_string().as_str())?;
                }
            }
            if empty > 0 {
                f.write_str(&empty.to_string())?;
            }
            if c < 7 {
                f.write_char('/')?;
            }
        }
        f.write_char(' ')?;
        f.write_str(if self.white_to_move { "w" } else { "b" })?;
        f.write_char(' ')?;
        if self.castling & 8 > 0 {
            f.write_char('K')?;
        }
        if self.castling & 4 > 0 {
            f.write_char('Q')?;
        }
        if self.castling & 2 > 0 {
            f.write_char('k')?;
        }
        if self.castling & 1 > 0 {
            f.write_char('q')?;
        }
        f.write_char(' ')?;
        f.write_char('-')?;
        f.write_char(' ')?;
        f.write_char('0')?;
        f.write_char(' ')?;
        f.write_char('1')?;
        Ok(())
    }
}
