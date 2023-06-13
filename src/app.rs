use crate::{
    board::{Move, ParsingError, Position},
    console::{Command, Console, CMD_PREFIX},
};
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
    pub searching: bool,
    pub piece_to_grab: Option<Position>,
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
            searching: false,
        }
    }

    pub async fn set_position(&mut self, fen: String) {
        match Board::from_fen(fen.clone()) {
            Ok(b) => {
                self.board = b;
                self.engine.set_position(fen.as_str()).await.unwrap();
                self.restart_search().await.unwrap();
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
                self.set_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq".to_string())
                    .await
            }
            _ => {}
        }
    }

    pub fn on_escape(&mut self) {
        self.reset_console();
        self.in_console = false;
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
                Ok(_) => self.searching = true,
                Err(err) => self.console.log_line(format!("err: {}", err)),
            },
            Command::StopSearch => match self.engine.stop().await {
                Ok(_) => self.searching = false,
                Err(err) => self.console.log_line(format!("err: {}", err)),
            },
            Command::AlgebraicNotation(mov) => match parse_algebraic_move(mov) {
                Ok(mov) if mov.in_bounds(self.board) => {
                    if let Err(err) = self.board.make_move(mov) {
                        self.console.log_line(format!("err: {}", err));
                    };
                }
                Ok(mov) => {
                    self.console
                        .log_line(format!("err: invalid move: {:?}", mov));
                }
                Err(err) => self.console.log_line(format!("err: {}", err)),
            },
        }
    }

    pub async fn on_mouse(&mut self, event: MouseEvent) {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let pos = get_relative_positions(event);
                match pos {
                    Some(p) => self.piece_to_grab = Some(p),
                    None => {}
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                let pos = match get_relative_positions(event) {
                    Some(p) => p,
                    None => return, // out of bounds
                };
                match self.piece_to_grab {
                    Some(p) if p == pos => {
                        if self.board.has_grabbed_piece() && self.board.in_bounds(p) {
                            self.drop_piece(p).await;
                        } else {
                            if let Err(_) = self.board.grab_piece(p) {
                                // tried to grab a piece that is not there
                            };
                        }
                    }
                    Some(p) => {
                        if let Ok(_) = self.board.grab_piece(p) {
                            if self.board.in_bounds(pos) {
                                self.drop_piece(pos).await;
                            }
                        };
                    }
                    None => {}
                }
                self.piece_to_grab = None;
            }
            _ => {}
        }
    }

    async fn drop_piece(&mut self, pos: Position) {
        match self.board.drop_piece(pos) {
            Ok(_) => {
                self.engine
                    .set_position(self.board.as_fen().as_str())
                    .await
                    .unwrap(); // FIXME: can panic for invalid positions
                self.restart_search().await.unwrap();
            }
            Err(err) => self.console.log_line(format!("err: {}", err)),
        };
    }

    async fn restart_search(&mut self) -> Result<()> {
        if self.searching {
            self.engine.stop().await?;
            self.engine.go_infinite().await?;
        }
        Ok(())
    }
}

/// Get the clicked position relative to the board.
fn get_relative_positions(event: MouseEvent) -> Option<Position> {
    // tui-rs makes it dificult to calculate the position of a mouse click relative to a widget
    // the workaround is knowing that the board always starts at the same absolute position in the screen (x=1, y=3)
    // and the squares have a fixed size (4w 1h).
    if event.column < 1 || event.row < 3 || event.column > 33 || event.row > 19 {
        return None;
    }
    if let Some(col) = event.column.checked_sub(1) {
        let col = col / 4;
        if let Some(row) = event.row.checked_sub(2) {
            let row = row / 2;
            if let Some(row) = row.checked_sub(1) {
                return Some(Position::Relative {
                    col: col as u8,
                    row: row as u8,
                });
            }
        }
    }
    None
}

/// Parse long algebraic notation move. i.e. e2e4
fn parse_algebraic_move(mov: String) -> Result<Move> {
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
        return Err(ParsingError::MoveParsingError.into());
    }
    Ok(Move::new(
        Position::Algebraic {
            rank: values[0],
            file: values[1],
        },
        Position::Algebraic {
            rank: values[2],
            file: values[3],
        },
    ))
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
