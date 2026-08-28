use std::{fmt::Display, path::PathBuf};

#[cfg(target_os = "macos")]
use crossterm::event::KeyModifiers;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::List,
};

use nomi::{core::read_entries, error::Result};

use crate::{
    pluralise,
    ui::{Panel, StatusBar, TextBuffer, TextInput, display_path, inner_area},
};

#[cfg(target_os = "macos")]
const MODIFIER_KEY: KeyModifiers = KeyModifiers::SUPER;

#[cfg(not(target_os = "macos"))]
const MODIFIER_KEY: KeyModifiers = KeyModifiers::CONTROL;

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
enum Focus {
    #[default]
    Pattern,
    Replacement,
    FileList,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Pattern => Focus::Replacement,
            Focus::Replacement => Focus::FileList,
            Focus::FileList => Focus::Pattern,
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
enum MatchMode {
    #[default]
    Literal,
    Regex,
}

impl MatchMode {
    pub fn toggle(self) -> Self {
        match self {
            MatchMode::Literal => MatchMode::Regex,
            MatchMode::Regex => MatchMode::Literal,
        }
    }

    pub fn colour(&self) -> Color {
        match self {
            MatchMode::Literal => Color::Blue,
            MatchMode::Regex => Color::Magenta,
        }
    }
}

impl Display for MatchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MatchMode::Literal => "Literal",
            MatchMode::Regex => "Regex",
        })
    }
}

#[derive(Debug)]
pub struct App {
    directory: PathBuf,
    entries: Vec<String>,
    pattern: TextBuffer,
    replacement: TextBuffer,
    focus: Focus,
    match_mode: MatchMode,
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
            focus: Focus::default(),
            match_mode: MatchMode::default(),
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
            .right_title(Line::from(vec![
                Span::styled(" ● ", self.match_mode.colour()),
                Span::styled(
                    format!("{} ", self.match_mode),
                    Style::default().fg(Color::Reset),
                ),
            ]))
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
                format!(" {} ", display_path(self.directory.as_path())),
            )
            .right_title(format!(
                " {} ",
                pluralise!(self.entries.len(), "item", "items")
            ))
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
        if key.modifiers.contains(MODIFIER_KEY) {
            match key.code {
                KeyCode::Char('r') => self.match_mode = self.match_mode.toggle(),
                _ => self.handle_focused_key(key),
            }
        } else {
            match key.code {
                KeyCode::Esc => self.exit = true,
                KeyCode::Tab => self.focus = self.focus.next(),
                _ => self.handle_focused_key(key),
            }
        }
    }

    fn handle_focused_key(&mut self, key: KeyEvent) {
        match self.focus {
            Focus::Pattern => self.pattern.handle_key(key),
            Focus::Replacement => self.replacement.handle_key(key),
            Focus::FileList => {}
        }
    }
}
