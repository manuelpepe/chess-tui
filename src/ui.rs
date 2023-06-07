use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::{app::App, board::Board};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(2 * 8 + 1)].as_ref())
        .split(f.size());

    draw_menu(f, app, chunks[0]);
    match app.tabs.index {
        0 => draw_board(f, app, chunks[1]),
        1 => draw_help(f, app, chunks[1]),
        _ => {}
    }
    // draw_console(f, app, chunks[2]);
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

pub fn draw_help<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let block = Block::default().title("Help").borders(Borders::ALL);
    let text = vec![
        Spans::from(vec![
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" - Quit\n"),
        ]),
        Spans::from(vec![
            Span::styled("<TAB>", Style::default().fg(Color::Yellow)),
            Span::raw(" - Next window"),
        ]),
    ];
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area)
}
