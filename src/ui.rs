use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::{app::App, board::Board};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(2 * 8 + 1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    draw_menu(f, app, chunks[0]);
    draw_board(f, app, chunks[1]);
    draw_console(f, app, chunks[2]);
}

pub fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        app.title.clone(),
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let titles = app
        .tabs
        .titles
        .iter()
        .cloned()
        .map(|s| Spans::from(s.to_string()))
        .collect();
    let tabs = Tabs::new(titles)
        .block(block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(DOT)
        .select(app.tabs.index);
    f.render_widget(tabs, area);
}

pub fn draw_board<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let board = Board::new();
    f.render_widget(board, area);
}

pub fn draw_console<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    return;
}
