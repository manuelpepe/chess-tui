use crate::app::App;
use anyhow::Result;
use app::NoopEngine;
use clap::Parser;
use cli::CLIArgs;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};
use tokio::task::yield_now;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use async_uci::engine::{ChessEngine, Engine};

mod app;
mod board;
mod cli;
mod console;
mod piece;
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

    let mut app = match args.engine_path {
        Some(path) => {
            let engine = get_engine(path).await?;
            let app = App::new(Box::leak(Box::new(engine)));
            app
        }
        None => {
            let engine = NoopEngine {};
            let app = App::new(Box::leak(Box::new(engine)));
            app
        }
    };
    app.set_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())
        .await;

    let mut terminal = init_terminal()?;
    let res = run_app(&mut terminal, app, tick_rate).await;
    close_terminal(&mut terminal)?;

    if let Err(err) = res {
        println!("ERR: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App<'_>,
    tick_rate: Duration,
) -> Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

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
                    _ => {}
                },
                Event::Mouse(event) => app.on_mouse(event).await,
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
