use anyhow::{bail, Result};
use thiserror::Error;
use tui::style::Style;
use tui_textarea::TextArea;

const CMD_PREFIX: &str = "> ";

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
}

impl Console {
    pub fn new() -> Console {
        Console {
            log: TextArea::default(),
            console: new_console(),
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

    pub fn set_cursor_style(&mut self, style: Style) {
        self.console.set_cursor_style(style);
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
        Command::from_string(command)
    }
}

#[derive(Debug, Clone, Copy, Error)]
pub enum CommandError {
    #[error("no command received")]
    NoCommand,

    #[error("invalid command")]
    InvalidCommand,
}

#[derive(Debug, Clone)]
pub enum Command {
    Exit,
    SetPosition(String),
    StartSeach,
    StopSearch,
}

impl Command {
    pub fn from_string(command: String) -> Result<Command> {
        let ch = match command.chars().next() {
            Some(c) => c,
            None => bail!(CommandError::NoCommand),
        };
        if let Some(cmd) = match ch {
            '!' => Some(Command::SetPosition(command[1..].to_string())),
            _ => None,
        } {
            return Ok(cmd);
        };
        let word = match command.split_whitespace().next() {
            Some(w) => w,
            None => bail!(CommandError::NoCommand),
        };
        if let Some(cmd) = match word {
            "exit" => Some(Command::Exit),
            ":set-position" => Some(Command::SetPosition(command[13..].to_string())),
            ":search" => Some(Command::StartSeach),
            ":stop" => Some(Command::StopSearch),
            _ => None,
        } {
            return Ok(cmd);
        };
        Err(CommandError::InvalidCommand.into())
    }
}
