use crate::console::{Command, Console};
use anyhow::Result;
use async_trait::async_trait;
use async_uci::engine::{ChessEngine, EngineOption, Evaluation};
use tui::style::{Color, Style};
use tui_textarea::CursorMove;

use crate::board::Board;

pub struct App<'a> {
    pub title: String,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub board: Board,
    pub console: Console,
    pub in_console: bool,
    pub engine: &'a mut dyn ChessEngine,
    pub last_engine_eval: Evaluation,
}

impl<'a> App<'a> {
    pub fn new(engine: &'a mut dyn ChessEngine) -> App<'a> {
        App {
            title: "Chess TUI".to_string(),
            should_quit: false,
            tabs: TabsState::new(vec!["Board", "Console", "Help"]),
            board: Board::new(),
            console: Console::new(),
            in_console: false,
            engine: engine,
            last_engine_eval: Evaluation::default(),
        }
    }

    pub async fn set_position(&mut self, fen: String) {
        match Board::from_fen(fen.clone()) {
            Ok(b) => {
                self.board = b;
                self.engine.set_position(fen.as_str()).await.unwrap();
            }
            Err(err) => self
                .console
                .log_line(format!("err: invalid position: {}", err)),
        }
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

    pub async fn on_tick(&mut self) {
        if let Some(ev) = self.engine.get_evaluation().await {
            if ev != self.last_engine_eval {
                self.console.log_line(format!("eval: {}", ev));
                self.last_engine_eval = ev;
            }
        };
        return;
    }

    pub async fn on_key(&mut self, c: char) {
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
            'S' => {
                self.set_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".to_string())
                    .await
            }
            _ => {}
        }
    }

    pub async fn on_enter(&mut self) {
        if self.in_console {
            match self.console.parse_command() {
                Ok(cmd) => self.exec_command(cmd).await,
                Err(err) => self.console.log_line(format!("err: {}", err)),
            };
            self.reset_console();
        }
    }

    pub fn on_backspace(&mut self) {
        if self.in_console {
            self.console.console.delete_char();
        }
    }

    pub fn on_delete(&mut self) {
        if self.in_console {
            self.console.console.delete_next_char();
        }
    }

    pub fn on_left(&mut self) {
        if self.in_console {
            self.console.console.move_cursor(CursorMove::Back)
        }
    }

    pub fn on_right(&mut self) {
        if self.in_console {
            self.console.console.move_cursor(CursorMove::Forward)
        }
    }

    pub async fn exec_command(&mut self, cmd: Command) {
        match cmd {
            Command::SetPosition(pos) => self.set_position(pos).await,
            Command::Exit => self.should_quit = true,
            Command::StartSeach => match self.engine.go_infinite().await {
                Ok(_) => {}
                Err(err) => self.console.log_line(format!("err: {}", err)),
            },
            Command::StopSearch => match self.engine.stop().await {
                Ok(_) => {}
                Err(err) => self.console.log_line(format!("err: {}", err)),
            },
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

pub struct NoopEngine {}

#[async_trait]
impl ChessEngine for NoopEngine {
    async fn start_uci(&mut self) -> Result<()> {
        Ok(())
    }

    async fn new_game(&mut self) -> Result<()> {
        Ok(())
    }

    async fn set_position(&mut self, _position: &str) -> Result<()> {
        Ok(())
    }

    async fn go_infinite(&mut self) -> Result<()> {
        Ok(())
    }

    async fn go_depth(&mut self, _plies: usize) -> Result<()> {
        Ok(())
    }

    async fn go_time(&mut self, _ms: usize) -> Result<()> {
        Ok(())
    }

    async fn go_mate(&mut self, _mate_in: usize) -> Result<()> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    async fn get_evaluation(&mut self) -> Option<Evaluation> {
        None
    }

    async fn get_options(&mut self) -> Result<Vec<EngineOption>> {
        Ok(Vec::new())
    }

    async fn set_option(&mut self, _option: String, _value: String) -> Result<()> {
        Ok(())
    }
}
