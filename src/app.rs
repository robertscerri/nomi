use std::{path::PathBuf, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::Backend};

use nomi::{
    error::NomiError,
    rename::{self, Entry, MatchMode, Preview},
};

use crate::ui;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Focus {
    Pattern,
    Replacement,
    Files,
}

pub struct App {
    pub directory: PathBuf,
    pub entries: Vec<Entry>,
    pub pattern: String,
    pub replacement: String,
    pub mode: MatchMode,
    pub focus: Focus,
    pub cursor: usize,
    pub scroll: usize,
    pub confirm: bool,
    pub message: Option<String>,
    should_quit: bool,
}

impl App {
    pub fn new(directory: PathBuf) -> Result<Self, NomiError> {
        let directory = directory.canonicalize()?;
        let entries = rename::read_entries(&directory)?;
        Ok(Self {
            directory,
            entries,
            pattern: String::new(),
            replacement: String::new(),
            mode: MatchMode::Regex,
            focus: Focus::Pattern,
            cursor: 0,
            scroll: 0,
            confirm: false,
            message: None,
            should_quit: false,
        })
    }

    pub fn preview(&self) -> Preview {
        rename::build_preview(
            &self.directory,
            &self.entries,
            &self.pattern,
            &self.replacement,
            self.mode,
        )
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), NomiError> {
        while !self.should_quit {
            terminal.draw(|frame| ui::draw(frame, self))?;
            if event::poll(Duration::from_millis(250))?
                && let Event::Key(key) = event::read()?
            {
                self.handle_key(key)?;
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<(), NomiError> {
        self.message = None;
        if self.confirm {
            return self.handle_confirmation(key);
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('r') => {
                    self.mode = match self.mode {
                        MatchMode::Literal => MatchMode::Regex,
                        MatchMode::Regex => MatchMode::Literal,
                    };
                    return Ok(());
                }
                KeyCode::Char('a') => {
                    let all_selected = self.entries.iter().all(|entry| entry.selected);
                    for entry in &mut self.entries {
                        entry.selected = !all_selected;
                    }
                    return Ok(());
                }
                KeyCode::Char('c') => {
                    self.should_quit = true;
                    return Ok(());
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Tab => self.next_focus(),
            KeyCode::BackTab => self.previous_focus(),
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Enter => {
                let preview = self.preview();
                if let Some(error) = preview.error {
                    self.message = Some(error);
                } else if preview.operations.is_empty() {
                    self.message = Some("Nothing to rename".into());
                } else {
                    self.confirm = true;
                }
            }
            _ if self.focus == Focus::Files => self.handle_file_key(key),
            _ => self.handle_input_key(key),
        }
        Ok(())
    }

    fn handle_confirmation(&mut self, key: KeyEvent) -> Result<(), NomiError> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                let preview = self.preview();
                rename::execute(&self.directory, &preview.operations)?;
                let count = preview.operations.len();
                self.entries = rename::read_entries(&self.directory)?;
                self.cursor = self.cursor.min(self.entries.len().saturating_sub(1));
                self.confirm = false;
                self.message = Some(format!("Renamed {count} item(s)"));
                self.pattern.clear();
                self.replacement.clear();
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.confirm = false;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_file_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.cursor = (self.cursor + 1).min(self.entries.len().saturating_sub(1));
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.entries.len().saturating_sub(1),
            KeyCode::Char(' ') => {
                if let Some(entry) = self.entries.get_mut(self.cursor) {
                    entry.selected = !entry.selected;
                }
            }
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn handle_input_key(&mut self, key: KeyEvent) {
        let input = match self.focus {
            Focus::Pattern => &mut self.pattern,
            Focus::Replacement => &mut self.replacement,
            Focus::Files => return,
        };
        match key.code {
            KeyCode::Char(character) => input.push(character),
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Delete => input.clear(),
            _ => {}
        }
    }

    fn next_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Pattern => Focus::Replacement,
            Focus::Replacement => Focus::Files,
            Focus::Files => Focus::Pattern,
        };
    }

    fn previous_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Pattern => Focus::Files,
            Focus::Replacement => Focus::Pattern,
            Focus::Files => Focus::Replacement,
        };
    }
}
