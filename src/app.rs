use crate::console::Console;
use tui::style::{Color, Style};

use crate::board::Board;
use crate::console::new_console;

pub struct App<'a> {
    pub title: String,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub board: Board,
    pub console: Console,
    pub in_console: bool,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            title: "Chess TUI".to_string(),
            should_quit: false,
            tabs: TabsState::new(vec!["Board", "Console", "Help"]),
            board: Board::new(),
            console: Console::new(),
            in_console: false,
        }
    }

    pub fn set_position(&mut self, fen: String) {
        self.board = Board::from_fen(fen)
    }

    pub fn reset_console(&mut self) {
        self.console.reset();
        self.in_console = false;
    }

    pub fn on_next_tab(&mut self) {
        self.tabs.next();
    }

    pub fn on_prev_tab(&mut self) {
        self.tabs.previous();
    }

    pub fn on_tick(&mut self) {
        return;
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            _ if self.in_console => {
                self.console.insert_char(c);
            }
            'q' => self.should_quit = true,
            ':' => {
                self.in_console = true;
                self.console
                    .set_cursor_style(Style::default().bg(Color::White));
            }
            'p' => self.set_position(
                "r2qk2r/pp3ppp/B1nbpn2/2pp1b2/Q2P1B2/2P1PN2/PP1N1PPP/R3K2R b KQkq - 4 8"
                    .to_string(),
            ),
            _ => {}
        }
    }

    pub fn on_enter(&mut self) {
        if self.in_console {
            self.console.eval_command();
            self.reset_console();
        }
    }
}

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}
