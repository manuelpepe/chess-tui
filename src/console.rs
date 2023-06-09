use tui::style::Style;
use tui_textarea::TextArea;

pub fn new_console() -> TextArea<'static> {
    let mut ta = TextArea::default();
    ta.set_cursor_line_style(Style::default());
    ta.set_cursor_style(Style::default());
    ta.insert_str("> ");
    ta
}

pub struct Console {
    pub log: TextArea<'static>,
    pub console: TextArea<'static>,
}

impl Console {
    pub fn new() -> Console {
        Console {
            log: TextArea::default(),
            console: new_console(),
        }
    }

    pub fn reset(&mut self) {
        self.console = new_console();
    }

    pub fn insert_char(&mut self, c: char) {
        self.console.insert_char(c);
    }

    pub fn set_cursor_style(&mut self, style: Style) {
        self.console.set_cursor_style(style);
    }

    pub fn eval_command(&mut self) {
        let command = self.console.lines().last().unwrap().to_string();
        self.log.insert_str(command);
        self.log.insert_newline();
    }
}
