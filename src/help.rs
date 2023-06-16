use std::iter;
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::Widget,
};
use tui_textarea::{Scrolling, TextArea};

pub struct HelpWindow {
    textarea: TextArea<'static>,
}

impl HelpWindow {
    pub fn new() -> Self {
        let help = get_help();
        let mut ta = TextArea::default();
        ta.set_cursor_line_style(Style::default());
        ta.set_cursor_style(Style::default());
        for line in help.iter() {
            ta.insert_str(line.to_string());
            ta.insert_newline();
        }
        ta.scroll(Scrolling::Delta {
            rows: -(help.len() as i16),
            cols: 0,
        });
        Self { textarea: ta }
    }

    pub fn scroll(&mut self, scroll: impl Into<Scrolling>) {
        self.textarea.scroll(scroll);
    }

    pub fn widget(&mut self) -> impl Widget + '_ {
        self.textarea.widget()
    }
}

fn get_help() -> Vec<String> {
    // TODO: Refactor this. It would be nice if we could restore the styles to the help text.
    // Otherwise this should be simplified to just a vector of strings.
    let shortcuts = [
        ("<TAB>", "Next window"),
        ("<UP/DOWN> or k/j or MouseWheel", "Scroll"),
        ("F2", "Toggle mouse capture"),
        (":", "Enter console and buffer with :"),
        ("!", "Enter console and buffer with !"),
        ("M", "Open legal moves pane"),
        ("H", "Open move history pane"),
        ("S", "Set starting position on the board"),
        ("q", "Quit"),
    ];
    let console_shortcuts = [
        ("<ESC>", "Exit console"),
        ("<Enter>", "Execute command"),
        ("<LEFT/RIGHT>", "Move cursor"),
        ("<UP/DOWN>", "Traverse command history"),
        ("!fen", "Print current position as FEN in the console (use F2 to toggle mouse capture and copy it)"),
        (":fen <fen>", "Set position on the board"),
        (
            ":move <mv>",
            "Play move on the board. Long algebraic notation used (i.e. e2e4)",
        ),
        (":search", "Start searching for best move"),
        (":stop", "Stop searching for best move"),
        (":flipboard", "Flip board vertically"),
        (":passturn", "Pass current player turn"),
        (":q", "Quit"),
    ];
    let legal_moves_shortcuts = [
        ("M", "Close legal moves pane"),
        ("<UP/DOWN> or k/j", "Change selected move"),
        ("(wip) <RIGHT/LEFT> or l/h", "Open or close group"),
        ("(wip) <ENTER>", "Make move on the board"),
        ("(wip) G", "Toggle move grouping"),
    ];
    let shortcuts_help: Vec<Spans> = shortcuts
        .iter()
        .map(|(k, v)| {
            Spans::from(vec![
                Span::styled(format!("  {}", k), Style::default().fg(Color::Yellow)),
                Span::raw(" - "),
                Span::raw(v.to_owned()),
            ])
        })
        .collect();
    let console_shortcuts_help: Vec<Spans> = console_shortcuts
        .iter()
        .map(|(k, v)| {
            Spans::from(vec![
                Span::styled(format!("  {}", k), Style::default().fg(Color::Yellow)),
                Span::raw(" - "),
                Span::raw(v.to_owned()),
            ])
        })
        .collect();
    let legal_moves_shortcuts_help: Vec<Spans> = legal_moves_shortcuts
        .iter()
        .map(|(k, v)| {
            Spans::from(vec![
                Span::styled(format!("  {}", k), Style::default().fg(Color::Yellow)),
                Span::raw(" - "),
                Span::raw(v.to_owned()),
            ])
        })
        .collect();
    let mut text = Vec::new();
    text.extend(iter::once(Spans::from(
        "<< Scroll with UP/DOWN/j/k/MouseWheel >>",
    )));
    text.extend(iter::once(Spans::from("")));
    text.extend(iter::once(Spans::from("General:")));
    text.extend(shortcuts_help);
    text.extend(iter::once(Spans::from("")));
    text.extend(iter::once(Spans::from("Console:")));
    text.extend(console_shortcuts_help);
    text.extend(iter::once(Spans::from("")));
    text.extend(iter::once(Spans::from("Legal Moves:")));
    text.extend(legal_moves_shortcuts_help);
    text.iter().map(|s| s.clone().into()).collect()
}
