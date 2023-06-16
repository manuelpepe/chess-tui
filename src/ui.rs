use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};
use tui_tree_widget::Tree;

use crate::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(2 * 8 + 1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());
    draw_menu(f, app, chunks[0]);
    match app.tabs.index {
        0 => draw_board(f, app, chunks[1]),
        1 => draw_console_log(f, app, chunks[1]),
        2 => draw_help(f, app, chunks[1]),
        _ => {}
    }
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
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(34), Constraint::Min(10)].as_ref())
        .split(area);
    let board_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(18), Constraint::Min(10)].as_ref())
        .split(chunks[0])[0];
    f.render_widget(app.board.clone(), board_chunk);
    draw_game_info(f, app, chunks[1])
}

pub fn draw_game_info<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    if app.in_moves_tree {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
            .split(area);
        draw_evaluation(f, app, chunks[0]);
        draw_moves_tree(f, app, chunks[1]);
    } else {
        draw_evaluation(f, app, area);
    }
}

pub fn draw_evaluation<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let block = Block::default()
        .title("Engine Evaluation")
        .borders(Borders::ALL);
    let mut text = wrap_text(format!("{}", app.last_engine_eval), area.width as usize - 2);
    text.push(Spans::from(""));
    text.extend(
        wrap_text(
            format!("Best: {}", app.last_engine_eval.pv.join(", ")),
            area.width as usize - 2,
        )
        .into_iter(),
    );
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

pub fn draw_moves_tree<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let items = Tree::new(app.moves_tree.items.clone())
        .block(Block::default().borders(Borders::ALL).title("Legal Moves"))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(items, area, &mut app.moves_tree.state);
}

pub fn draw_console_log<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    f.render_widget(app.console.log.widget(), area);
}

pub fn draw_console<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let block = Block::default().title("Console").borders(Borders::ALL);
    app.console.console.set_block(block);
    let widget = app.console.console.widget();
    f.render_widget(widget, area)
}

pub fn draw_help<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    f.render_widget(app.help.widget(), area);
}

fn wrap_text(text: String, width: usize) -> Vec<Spans<'static>> {
    text.chars()
        .collect::<Vec<_>>()
        .chunks(width)
        .map(|c| Spans::from(c.iter().collect::<String>()))
        .collect::<Vec<_>>()
}
