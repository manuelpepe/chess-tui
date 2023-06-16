use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, Stdout, Write},
    time::{Duration, Instant},
};
use tokio::task::yield_now;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::app::{App, NoopEngine};
use crate::cli::CLIArgs;
use async_uci::engine::{ChessEngine, Engine};

mod app;
mod board;
mod cli;
mod console;
mod fen;
mod help;
mod piece;
mod tree;
mod ui;

async fn get_engine(path: String) -> Result<Engine> {
    let mut eng = Engine::new(path.as_str()).await?;
    eng.start_uci().await?;
    Ok(eng)
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn close_terminal(term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    term.show_cursor()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let tick_rate = Duration::from_millis(args.tickrate);

    let app = match args.engine_path {
        Some(path) => {
            let engine = get_engine(path).await?;
            let leaked_engine = Box::leak(Box::new(engine));
            let app = App::new(leaked_engine).unwrap();
            app
        }
        None => {
            let engine = NoopEngine {};
            let leaked_engine = Box::leak(Box::new(engine));
            let app = App::new(leaked_engine).unwrap();
            app
        }
    };

    let mut terminal = init_terminal()?;
    let res = run_app(&mut terminal, app, tick_rate).await;
    close_terminal(&mut terminal)?;

    if let Err(err) = res {
        println!("ERR: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend + Write>(
    terminal: &mut Terminal<B>,
    mut app: App<'_>,
    tick_rate: Duration,
) -> Result<()> {
    let mut last_tick = Instant::now();
    let mut mouse_captured = true;
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char(c) => app.on_key(c).await,
                    KeyCode::BackTab => app.on_prev_tab(),
                    KeyCode::Tab => app.on_next_tab(),
                    KeyCode::Esc => app.on_escape(),
                    KeyCode::Enter => app.on_enter().await,
                    KeyCode::Backspace => app.on_backspace(),
                    KeyCode::Delete => app.on_delete(),
                    KeyCode::Left => app.on_left(),
                    KeyCode::Right => app.on_right(),
                    KeyCode::Up => app.on_up(),
                    KeyCode::Down => app.on_down(),
                    KeyCode::F(2) => {
                        if mouse_captured {
                            execute!(terminal.backend_mut(), DisableMouseCapture)?;
                            mouse_captured = false;
                        } else {
                            execute!(terminal.backend_mut(), EnableMouseCapture)?;
                            mouse_captured = true;
                        }
                    }
                    _ => {}
                },
                Event::Mouse(event) => match event.kind {
                    MouseEventKind::Up(_) | MouseEventKind::Down(_) => app.on_mouse(event).await,
                    MouseEventKind::ScrollDown => app.on_down(),
                    MouseEventKind::ScrollUp => app.on_up(),
                    _ => {}
                },
                _ => {}
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await;
            last_tick = Instant::now();
        }
        if app.should_quit {
            return Ok(());
        }
        yield_now().await;
    }
}
