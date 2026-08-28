use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    widgets::List,
};

use nomi::{core::read_entries, error::Result};

use crate::ui::{Panel, StatusBar, TextBuffer, TextInput, inner_area};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Focus {
    Pattern,
    Replacement,
    FileList,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Self::Pattern => Self::Replacement,
            Self::Replacement => Self::FileList,
            Self::FileList => Self::Pattern,
        }
    }
}

#[derive(Debug)]
pub struct App {
    directory: PathBuf,
    entries: Vec<String>,
    pattern: TextBuffer,
    replacement: TextBuffer,
    focus: Focus,
    exit: bool,
}

impl App {
    pub fn try_new(directory: PathBuf) -> Result<Self> {
        let directory = directory.canonicalize()?;
        let entries = read_entries(&directory)?;

        Ok(Self {
            directory,
            entries,
            pattern: TextBuffer::default(),
            replacement: TextBuffer::default(),
            focus: Focus::Pattern,
            exit: false,
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let [pattern_area, replacement_area, file_list_area, status_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(frame.area());

        frame.render_widget(
            Panel::new(
                TextInput::new(self.pattern.value(), self.pattern.cursor()),
                " Pattern ",
            )
            .focused(self.focus == Focus::Pattern),
            pattern_area,
        );

        frame.render_widget(
            Panel::new(
                TextInput::new(self.replacement.value(), self.replacement.cursor()),
                " Replacement ",
            )
            .focused(self.focus == Focus::Replacement),
            replacement_area,
        );

        frame.render_widget(
            Panel::new(
                List::new(self.entries.iter().map(String::as_str)),
                format!(" {} ", self.directory.display()),
            )
            .right_title(format!(" {} items ", self.entries.len()))
            .focused(self.focus == Focus::FileList),
            file_list_area,
        );

        frame.render_widget(StatusBar::new(), status_area);

        match self.focus {
            Focus::Pattern => self.set_text_cursor(frame, pattern_area, &self.pattern),
            Focus::Replacement => self.set_text_cursor(frame, replacement_area, &self.replacement),
            Focus::FileList => {}
        }
    }

    fn set_text_cursor(&self, frame: &mut Frame, area: Rect, buffer: &TextBuffer) {
        let input = TextInput::new(buffer.value(), buffer.cursor());
        frame.set_cursor_position(input.cursor_position(inner_area(area)));
    }

    fn handle_events(&mut self) -> Result<()> {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()?
            && key.is_press()
        {
            self.handle_key(key);
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.exit = true,
            KeyCode::Tab => self.focus = self.focus.next(),
            _ => match self.focus {
                Focus::Pattern => self.pattern.handle_key(key.code),
                Focus::Replacement => self.replacement.handle_key(key.code),
                Focus::FileList => {}
            },
        }
    }
}
