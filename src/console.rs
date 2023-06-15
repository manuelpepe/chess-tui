use std::cmp::Ordering;

use anyhow::{bail, Result};
use thiserror::Error;
use tui::style::{Color, Style};
use tui_textarea::TextArea;

use crate::board::{Move, Position};

pub const CMD_PREFIX: &str = "> ";

pub fn new_console() -> TextArea<'static> {
    let mut ta = TextArea::default();
    ta.set_cursor_line_style(Style::default());
    ta.set_cursor_style(Style::default());
    ta.insert_str(CMD_PREFIX);
    ta
}

pub struct Console {
    pub log: TextArea<'static>,
    pub console: TextArea<'static>,
    pub history: Vec<String>,
    pub history_ix: usize,
}

impl Console {
    pub fn new() -> Console {
        Console {
            log: TextArea::default(),
            console: new_console(),
            history: Vec::new(),
            history_ix: 0,
        }
    }

    pub fn reset(&mut self) {
        self.console = new_console();
    }

    pub fn insert_char(&mut self, c: char) {
        self.console.insert_char(c);
    }

    pub fn log_line(&mut self, s: String) {
        self.log.insert_str(s);
        self.log.insert_newline();
    }

    pub fn set_active_cursor(&mut self) {
        self.console
            .set_cursor_style(Style::default().bg(Color::White));
    }

    pub fn parse_command(&mut self) -> Result<Command> {
        let command = self
            .console
            .lines()
            .last()
            .unwrap()
            .to_string()
            .strip_prefix(CMD_PREFIX)
            .unwrap()
            .to_string();
        self.log_line(command.clone());
        self.add_to_history(command.clone());
        Command::from_string(command)
    }

    pub fn add_to_history(&mut self, command: String) {
        self.history.push(command);
        self.history_ix = self.history.len();
    }

    pub fn move_history_forwards(&mut self) {
        let hist_len = self.history.len();
        if hist_len < 1 {
            return;
        }
        match self.history_ix.cmp(&(hist_len - 1)) {
            Ordering::Less => {
                self.history_ix += 1;
                self.reset();
                self.set_active_cursor();
                self.console
                    .insert_str(self.history[self.history_ix].clone());
            }
            Ordering::Equal => {
                self.history_ix += 1;
                self.reset();
                self.set_active_cursor();
            }
            Ordering::Greater => {
                panic!("history in the future!")
            }
        }
    }

    pub fn move_history_backwards(&mut self) {
        if self.history_ix > 0 {
            self.history_ix -= 1;
            self.reset();
            self.set_active_cursor();
            self.console
                .insert_str(self.history[self.history_ix].clone());
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum CommandError {
    #[error("no command received")]
    NoCommand,

    #[error("invalid command")]
    InvalidCommand,

    #[error("error parsing move: {mov}")]
    MoveParsingError { mov: String },
}

#[derive(Debug, Clone)]
pub enum Command {
    Exit,
    SetPosition(String),
    StartSeach,
    StopSearch,
    MakeMove(ParsedMove),
    PassTurn,
}

impl Command {
    pub fn from_string(command: String) -> Result<Self> {
        let ch_cmd = Self::parse_char_cmd(command.clone());
        if ch_cmd.is_ok() {
            return ch_cmd;
        }
        Self::parse_word_cmd(command)
    }

    fn parse_char_cmd(command: String) -> Result<Self> {
        let ch = match command.chars().next() {
            Some(c) => c,
            None => bail!(CommandError::NoCommand),
        };
        let cmd = match ch {
            '!' => Command::SetPosition(command[1..].to_string()),
            _ => bail!(CommandError::InvalidCommand),
        };
        Ok(cmd)
    }

    fn parse_word_cmd(command: String) -> Result<Self> {
        let word = match command.split_whitespace().next() {
            Some(w) => w,
            None => bail!(CommandError::NoCommand),
        };
        let cmd = match word {
            "exit" | ":q" => Command::Exit,
            ":passturn" => Command::PassTurn,
            ":search" => Command::StartSeach,
            ":stop" => Command::StopSearch,
            ":set-position" if command.len() > 13 => {
                Command::SetPosition(command[13..].to_string())
            }
            ":move" if command.len() > 6 => {
                let mov = parse_algebraic_move(command[6..].to_string())?;
                Command::MakeMove(mov)
            }
            _ => bail!(CommandError::InvalidCommand),
        };
        Ok(cmd)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ParsedMove {
    Basic { mov: Move },
    CastleLong,
    CastleShort,
}

/// Parse long algebraic notation move. i.e. e2e4
fn parse_algebraic_move(mov: String) -> Result<ParsedMove> {
    let mov = mov.trim();
    if mov == "0-0" || mov == "O-O" {
        return Ok(ParsedMove::CastleShort);
    }
    if mov == "0-0-0" || mov == "O-O-O" {
        return Ok(ParsedMove::CastleLong);
    }
    let values = mov
        .chars()
        .take(4)
        .filter_map(|c| match c {
            'a'..='h' => Some(c as u8 - 97),
            '1'..='8' => Some((c.to_digit(10).unwrap() - 1) as u8),
            _ => None,
        })
        .collect::<Vec<_>>();
    if values.len() != 4 {
        let mov = mov.to_string();
        return Err(CommandError::MoveParsingError { mov }.into());
    }
    Ok(ParsedMove::Basic {
        mov: Move::new(
            Position::Algebraic {
                rank: values[0],
                file: values[1],
            },
            Position::Algebraic {
                rank: values[2],
                file: values[3],
            },
        ),
    })
}
