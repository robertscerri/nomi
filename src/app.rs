use std::{fmt::Display, path::PathBuf};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
};

use nomi::{
    core::read_entries,
    error::{Error, Result},
};

use crate::{
    pluralise,
    rename::{Rename, RenameConfig},
    selection::Selection,
    ui::{
        FileList, MODIFIER_KEY, Panel, StatusBar, TextBuffer, TextInput, display_path, inner_area,
    },
};

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
pub enum MatchMode {
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
    entries: Selection<Rename>,
    pattern: TextBuffer,
    replacement: TextBuffer,
    focus: Focus,
    match_mode: MatchMode,
    preview_error: Option<Error>,
    confirming: bool,
    exit: bool,
}

impl App {
    pub fn try_new(directory: PathBuf) -> Result<Self> {
        let directory = directory.canonicalize()?;
        let entries = Selection::new(read_entries(&directory)?.into_iter().map(Rename::new));

        Ok(Self {
            directory,
            entries,
            pattern: TextBuffer::default(),
            replacement: TextBuffer::default(),
            focus: Focus::default(),
            match_mode: MatchMode::default(),
            preview_error: None,
            confirming: false,
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
            Constraint::Length(1),
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
                FileList::new(&self.entries, self.preview_error.as_ref(), self.confirming),
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
        if self.confirming {
            match key.code {
                KeyCode::Enter => self.execute_renames(),
                KeyCode::Esc => self.confirming = false,
                _ => {}
            }
            return;
        }

        if key.modifiers.contains(MODIFIER_KEY) {
            match key.code {
                KeyCode::Char('r') => {
                    self.match_mode = self.match_mode.toggle();
                    self.refresh_preview();
                }
                _ => self.handle_focused_key(key),
            }
        } else {
            match key.code {
                KeyCode::Esc => self.exit = true,
                KeyCode::Tab => self.focus = self.focus.next(),
                KeyCode::Enter if self.preview_error.is_none() && self.has_renames() => {
                    self.confirming = true
                }
                _ => self.handle_focused_key(key),
            }
        }
    }

    fn handle_focused_key(&mut self, key: KeyEvent) {
        match self.focus {
            Focus::Pattern => {
                self.pattern.handle_key(key);
                if changes_text(key.code) {
                    self.refresh_preview();
                }
            }
            Focus::Replacement => {
                self.replacement.handle_key(key);
                if changes_text(key.code) {
                    self.refresh_preview();
                }
            }
            Focus::FileList => FileList::handle_key(&mut self.entries, key),
        }
    }

    fn refresh_preview(&mut self) {
        match RenameConfig::new(
            self.pattern.value(),
            self.replacement.value(),
            self.match_mode,
        ) {
            Ok(config) => {
                for rename in self.entries.values_mut() {
                    rename.preview(&config);
                }
                self.preview_error = None;
            }
            Err(error) => self.preview_error = Some(error),
        }
    }

    fn has_renames(&self) -> bool {
        self.entries
            .selected_values()
            .any(|rename| rename.source() != rename.destination())
    }

    fn execute_renames(&mut self) {
        let result = self
            .entries
            .selected_values()
            .try_for_each(|rename| rename.execute(&self.directory));

        match read_entries(&self.directory) {
            Ok(entries) => {
                self.entries = Selection::new(entries.into_iter().map(Rename::new));
                self.refresh_preview();
                if let Err(error) = result {
                    self.preview_error = Some(error);
                }
            }
            Err(error) => self.preview_error = Some(error),
        }

        self.confirming = false;
    }
}

fn changes_text(key: KeyCode) -> bool {
    matches!(key, KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete)
}
