use crate::{
    board::{Board, Move, Position},
    console::{Command, Console, ParsedMove, CMD_PREFIX},
    help::HelpWindow,
    tree::StatefulTree,
};
use anyhow::Result;
use async_trait::async_trait;
use async_uci::engine::{ChessEngine, EngineOption, Evaluation};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use tui_textarea::CursorMove;
use tui_tree_widget::TreeItem;

pub const INITIAL_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq";

#[derive(Debug, PartialEq, Eq)]
pub enum SecondaryBoardPane {
    None,
    MovesTree,
    History,
}

pub struct App<'a> {
    pub title: String,
    pub should_quit: bool,

    pub board: Board,
    pub flipped_board: bool,

    pub console: Console,
    pub in_console_input: bool,

    pub engine: &'a mut dyn ChessEngine,
    pub last_engine_eval: Evaluation,
    pub searching: bool,

    pub piece_to_grab: Option<Position>,
    pub help: HelpWindow,

    pub tabs: TabsState<'a>,
    pub moves_tree: StatefulTree<'a>,
    pub history_tree: StatefulTree<'a>,

    pub secondary_pane: SecondaryBoardPane,
}

/// Functional Implementations
impl<'a> App<'a> {
    pub fn new(engine: &'a mut dyn ChessEngine) -> Result<App<'a>> {
        App::from_fen(engine, INITIAL_POSITION.to_string())
    }

    pub fn from_fen(engine: &'a mut dyn ChessEngine, fen: String) -> Result<App<'a>> {
        let mut app = App {
            title: "Chess TUI".to_string(),
            should_quit: false,
            tabs: TabsState::new(vec!["Board", "Console", "Help"]),
            board: Board::from_fen(fen)?,
            flipped_board: false,
            console: Console::new(),
            in_console_input: false,
            engine,
            last_engine_eval: Evaluation::default(),
            piece_to_grab: None,
            searching: false,
            moves_tree: StatefulTree::with_items(Vec::new()),
            history_tree: StatefulTree::with_items(Vec::new()),
            help: HelpWindow::new(),
            secondary_pane: SecondaryBoardPane::None,
        };
        app.update_trees();
        Ok(app)
    }

    fn update_trees(&mut self) {
        self.update_move_tree();
        self.update_history_tree();
    }

    fn update_move_tree(&mut self) {
        let moves = self.board.get_legal_moves();
        let items = moves
            .iter()
            .map(|m| TreeItem::new_leaf(format!("{}", m)))
            .collect::<Vec<_>>();
        self.moves_tree = StatefulTree::with_items(items);
    }

    fn update_history_tree(&mut self) {
        let history = self.board.get_history();
        let chunks = history.chunks_exact(2);
        let (len, remainder) = (chunks.len(), chunks.remainder());
        let mut items = chunks
            .enumerate()
            .map(|(ix, movs)| {
                TreeItem::new(
                    format!("{}. ", ix),
                    vec![
                        TreeItem::new_leaf(movs[0].to_string()),
                        TreeItem::new_leaf(movs[1].to_string()),
                    ],
                )
            })
            .collect::<Vec<_>>();
        if remainder.len() == 1 {
            let last = remainder[0];
            items.push(TreeItem::new(
                format!("{}. ", len),
                vec![TreeItem::new_leaf(last.to_string())],
            ));
        }
        self.history_tree = StatefulTree::with_items(items);
        self.history_tree.last();
    }

    async fn set_position(&mut self, fen: String) {
        match Board::from_fen(fen.clone()) {
            Ok(b) => {
                self.board = b;
                self.update_engine_position().await.unwrap();
                self.update_trees();
            }
            Err(err) => self
                .console
                .log_line(format!("err: invalid position: {}", err)),
        }
    }

    async fn drop_piece(&mut self, pos: Position) -> Result<()> {
        match self.board.drop_piece(pos) {
            Ok(_) => self.update_engine_position().await,
            Err(err) => {
                self.console.log_line(format!("err: {}", err));
                Err(err)
            }
        }
    }

    async fn update_engine_position(&mut self) -> Result<()> {
        let fen = self.board.as_fen();
        self.engine.set_position(fen.as_str()).await?;
        self.restart_search().await?;
        Ok(())
    }

    async fn restart_search(&mut self) -> Result<()> {
        if self.searching {
            self.engine.stop().await?;
            self.engine.go_infinite().await?;
        }
        Ok(())
    }

    fn focus_console(&mut self, buffered: char) {
        self.in_console_input = true;
        self.console.set_active_cursor();
        self.console.insert_char(buffered);
    }

    fn reset_console(&mut self) {
        self.console.reset();
        self.in_console_input = false;
    }

    fn flip_board(&mut self) {
        self.flipped_board = !self.flipped_board;
        self.board.set_flipped(self.flipped_board);
    }

    fn toggle_moves_tree(&mut self) {
        self.secondary_pane = match self.secondary_pane {
            SecondaryBoardPane::MovesTree => SecondaryBoardPane::None,
            _ => SecondaryBoardPane::MovesTree,
        }
    }

    fn toggle_history(&mut self) {
        self.secondary_pane = match self.secondary_pane {
            SecondaryBoardPane::History => SecondaryBoardPane::None,
            _ => SecondaryBoardPane::History,
        }
    }

    fn log_fen(&mut self) {
        self.console
            .log_line("FEN of current position:".to_string());
        self.console.log_line(self.board.as_fen());
    }
}

/// Trigger Implementations
impl<'a> App<'a> {
    pub async fn on_tick(&mut self) {
        if let Some(ev) = self.engine.get_evaluation().await {
            if ev != self.last_engine_eval {
                self.console.log_line(format!("eval: {}", ev));
                self.last_engine_eval = ev;
            }
        };
    }

    pub async fn on_enter(&mut self) {
        if self.in_console_input {
            match self.console.parse_command() {
                Ok(cmd) => self.on_command(cmd).await,
                Err(err) => self.console.log_line(format!("err: {}", err)),
            };
            self.reset_console();
        }
    }

    pub fn on_next_tab(&mut self) {
        self.tabs.next();
    }

    pub fn on_prev_tab(&mut self) {
        self.tabs.previous();
    }

    pub fn on_escape(&mut self) {
        if self.in_console_input {
            self.reset_console();
            self.in_console_input = false;
        }
    }

    pub fn on_backspace(&mut self) {
        if self.in_console_input && self.console.console.cursor().1 > CMD_PREFIX.len() {
            self.console.console.delete_char();
        }
    }

    pub fn on_delete(&mut self) {
        if self.in_console_input {
            self.console.console.delete_next_char();
        }
    }

    pub fn on_left(&mut self) {
        if self.in_console_input {
            self.console.console.move_cursor(CursorMove::Back);
            return;
        }
        match self.secondary_pane {
            SecondaryBoardPane::MovesTree => self.moves_tree.left(),
            SecondaryBoardPane::History => self.history_tree.left(),
            _ => {}
        }
    }

    pub fn on_right(&mut self) {
        if self.in_console_input {
            self.console.console.move_cursor(CursorMove::Forward);
            return;
        }
        match self.secondary_pane {
            SecondaryBoardPane::MovesTree => self.moves_tree.right(),
            SecondaryBoardPane::History => self.history_tree.right(),
            _ => {}
        }
    }

    pub fn on_up(&mut self) {
        if self.in_console_input {
            self.console.move_history_backwards();
            return;
        }
        match self.tabs.index {
            1 => self.console.scroll((-1, 0)),
            2 => self.help.scroll((-1, 0)),
            _ => {}
        }
        match self.secondary_pane {
            SecondaryBoardPane::MovesTree => self.moves_tree.up(),
            SecondaryBoardPane::History => self.history_tree.up(),
            _ => {}
        }
    }

    pub fn on_down(&mut self) {
        if self.in_console_input {
            self.console.move_history_forwards();
            return;
        }
        match self.tabs.index {
            1 => self.console.scroll((1, 0)),
            2 => self.help.scroll((1, 0)),
            _ => {}
        }
        match self.secondary_pane {
            SecondaryBoardPane::MovesTree => self.moves_tree.down(),
            SecondaryBoardPane::History => self.history_tree.down(),
            _ => {}
        }
    }

    pub async fn on_key(&mut self, c: char) {
        match c {
            _ if self.in_console_input => self.console.insert_char(c),
            'q' => self.should_quit = true,
            ':' => self.focus_console(':'),
            '!' => self.focus_console('!'),
            'S' => self.set_position(INITIAL_POSITION.to_string()).await,
            'M' => self.toggle_moves_tree(),
            'H' => self.toggle_history(),
            'k' => self.on_up(),
            'j' => self.on_down(),
            'h' => self.on_left(),
            'l' => self.on_right(),
            _ => {}
        }
    }

    pub async fn on_command(&mut self, cmd: Command) {
        match cmd {
            Command::Exit => self.should_quit = true,
            Command::SetPosition(pos) => self.set_position(pos).await,
            Command::StartSeach => match self.engine.go_infinite().await {
                Ok(_) => self.searching = true,
                Err(err) => self.console.log_line(format!("err: {}", err)),
            },
            Command::StopSearch => match self.engine.stop().await {
                Ok(_) => self.searching = false,
                Err(err) => self.console.log_line(format!("err: {}", err)),
            },
            Command::MakeMove(mov) => {
                let mov = match mov {
                    ParsedMove::Basic { mov } => mov,
                    ParsedMove::CastleShort => Move::castle_short(self.board.white_to_move()),
                    ParsedMove::CastleLong => Move::castle_long(self.board.white_to_move()),
                };
                if let Err(err) = self.board.make_move(mov) {
                    self.console.log_line(format!("err: {}", err));
                };
                self.update_engine_position().await.unwrap();
                self.update_trees();
            }
            Command::PassTurn => {
                self.board.pass_turn();
                self.update_trees();
            }
            Command::FlipBoard => self.flip_board(),
            Command::GetFen => self.log_fen(),
        }
    }

    pub async fn on_mouse(&mut self, event: MouseEvent) {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(p) = get_relative_positions(event, self.flipped_board) {
                    self.piece_to_grab = Some(p);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                let pos = match get_relative_positions(event, self.flipped_board) {
                    Some(p) => p,
                    None => return, // out of bounds
                };
                match self.piece_to_grab {
                    Some(p) if p == pos => {
                        if self.board.has_grabbed_piece() && self.board.in_bounds(p) {
                            if (self.drop_piece(p).await).is_ok() {
                                self.update_trees();
                            };
                        } else if self.board.grab_piece(p).is_err() {
                            // tried to grab a piece that is not there
                        }
                    }
                    Some(p) => {
                        if self.board.grab_piece(p).is_ok()
                            && self.board.in_bounds(pos)
                            && (self.drop_piece(pos).await).is_ok()
                        {
                            self.update_trees();
                        };
                    }
                    None => {}
                }
                self.piece_to_grab = None;
            }
            _ => {}
        }
    }
}

/// Get the clicked position relative to the board.
fn get_relative_positions(event: MouseEvent, flipped: bool) -> Option<Position> {
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
                    flip: flipped,
                });
            }
        }
    }
    None
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
