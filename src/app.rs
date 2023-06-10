use crate::console::{Command, Console, CMD_PREFIX};
use anyhow::Result;
use async_trait::async_trait;
use async_uci::engine::{ChessEngine, EngineOption, Evaluation};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
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
    pub piece_to_grab: Option<(u16, u16)>,
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
            piece_to_grab: None,
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
                self.console.set_active_cursor();
                self.console.insert_char(':');
            }
            '!' => {
                self.in_console = true;
                self.console.set_active_cursor();
                self.console.insert_char('!');
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
        if self.in_console && self.console.console.cursor().1 > CMD_PREFIX.len() {
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

    pub fn on_up(&mut self) {
        if self.in_console {
            self.console.move_history_backwards()
        }
    }

    pub fn on_down(&mut self) {
        if self.in_console {
            self.console.move_history_forwards()
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
            Command::AlgebraicNotation(mov) => match self.board.make_move(mov) {
                Ok((from, to)) => {
                    self.console.log_line(format!("from: {}, to: {}", from, to));
                }
                Err(err) => self.console.log_line(format!("err: {}", err)),
            },
        }
    }

    fn get_relative_positions(event: MouseEvent) -> Option<(u16, u16)> {
        // calculate column and row relative to board.
        // tui-rs makes it dificult to calculate the position of a mouse click relative to a widget
        // the workaround is knowing that the board always starts at the same absolute position in the screen (x=1, y=3)
        // and the squares have a fixed size (4w 1h, 1w padding).

        // TODO: refactor this into a more appropiate structure.
        //  probably an Enum with differnt types of moves (algebraic, relative, etc) with
        //  a method to get the 1d index in the board (instead of this x,y tuple).
        match event.column.checked_sub(1) {
            Some(col) => {
                let col = col / 5;
                match event.row.checked_sub(2) {
                    Some(row) => {
                        let row = (row / 2) - 1;
                        return Some((col, row));
                    }
                    None => None,
                }
            }
            None => None,
        }
    }

    pub fn on_mouse(&mut self, event: MouseEvent) {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let pos = App::get_relative_positions(event);
                match pos {
                    Some(p) => {
                        self.piece_to_grab = Some(p);
                        self.console.log_line(format!("grabed: {:?}", p));
                    }
                    None => {}
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                let pos = match App::get_relative_positions(event) {
                    Some(p) => p,
                    None => return,
                };
                match self.piece_to_grab {
                    Some(p) if p == pos => {
                        if self.board.has_grabbed_piece() {
                            let ix = p.0 + p.1 * 8;
                            self.board.drop_piece(ix as u8);
                            self.console.log_line(format!("drop: {:?} at {:?}", p, pos));
                        } else {
                            let ix = p.0 + p.1 * 8;
                            self.board.grab_piece(ix as u8);
                            self.console.log_line(format!("grab: {:?}", p));
                        }
                    }
                    Some(p) => {
                        let ix = p.0 + p.1 * 8;
                        self.board.grab_piece(ix as u8);
                        let newix = pos.0 + pos.1 * 8;
                        self.board.drop_piece(newix as u8);
                        self.console.log_line(format!("drop: {:?} at {:?}", p, pos));
                    }
                    None => {}
                }
                self.piece_to_grab = None;
                self.console.log_line(format!("mouse: {:?}", event));
            }
            _ => {}
        }
    }
}

/// Keeps the state of the tabs in the UI.
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

/// An engine that does nothing, used by default when the user does not provide an engine.
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
