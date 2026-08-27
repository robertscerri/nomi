use std::{path::PathBuf, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{Terminal, backend::Backend};

use nomi::{
    error::NomiError,
    rename::{self, Entry, MatchMode, RenamePreview},
};

use crate::ui;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Focus {
    Pattern,
    Replacement,
    Files,
}

#[derive(Default)]
pub struct TextInput {
    value: String,
    cursor: usize,
}

impl TextInput {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn insert(&mut self, character: char) {
        let byte_index = self.byte_index();
        self.value.insert(byte_index, character);
        self.cursor += 1;
    }

    fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }

        self.cursor -= 1;
        let byte_index = self.byte_index();
        self.value.remove(byte_index);
    }

    fn delete(&mut self) {
        if self.cursor < self.value.chars().count() {
            let byte_index = self.byte_index();
            self.value.remove(byte_index);
        }
    }

    fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.value.chars().count());
    }

    fn move_to_start(&mut self) {
        self.cursor = 0;
    }

    fn move_to_end(&mut self) {
        self.cursor = self.value.chars().count();
    }

    fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    fn byte_index(&self) -> usize {
        self.value
            .char_indices()
            .nth(self.cursor)
            .map_or(self.value.len(), |(index, _)| index)
    }
}

pub struct App {
    pub directory: PathBuf,
    pub entries: Vec<Entry>,
    pub pattern: TextInput,
    pub replacement: TextInput,
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
            pattern: TextInput::default(),
            replacement: TextInput::default(),
            mode: MatchMode::Regex,
            focus: Focus::Pattern,
            cursor: 0,
            scroll: 0,
            confirm: false,
            message: None,
            should_quit: false,
        })
    }

    pub fn preview(&self) -> RenamePreview {
        RenamePreview::build(
            &self.directory,
            &self.entries,
            self.pattern.value(),
            self.replacement.value(),
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
        if key.kind == KeyEventKind::Release {
            return Ok(());
        }

        self.message = None;

        if self.confirm {
            return self.handle_confirmation(key);
        }

        if shortcut_modifier(key.modifiers) {
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
                _ => return Ok(()),
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
                } else if preview.is_empty() {
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
                self.execute_preview()?;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.confirm = false;
            }
            _ => {}
        }

        Ok(())
    }

    fn execute_preview(&mut self) -> Result<(), NomiError> {
        let preview = self.preview();
        let count = preview.len();

        if let Err(error) = preview.execute(&self.directory) {
            if matches!(error, NomiError::Rollback { .. }) {
                return Err(error);
            }

            self.confirm = false;
            self.message = Some(error.to_string());
            return Ok(());
        }

        self.entries = rename::read_entries(&self.directory)?;
        self.clamp_cursor();
        self.confirm = false;
        self.message = Some(format!("Renamed {count} item(s)"));
        self.pattern.clear();
        self.replacement.clear();

        Ok(())
    }

    fn handle_file_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.entries.len().saturating_sub(1),
            KeyCode::Char(' ') => self.toggle_selected(),
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn select_previous(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn select_next(&mut self) {
        self.cursor = (self.cursor + 1).min(self.entries.len().saturating_sub(1));
    }

    fn toggle_selected(&mut self) {
        if let Some(entry) = self.entries.get_mut(self.cursor) {
            entry.selected = !entry.selected;
        }
    }

    fn clamp_cursor(&mut self) {
        self.cursor = self.cursor.min(self.entries.len().saturating_sub(1));
    }

    fn handle_input_key(&mut self, key: KeyEvent) {
        let input = match self.focus {
            Focus::Pattern => &mut self.pattern,
            Focus::Replacement => &mut self.replacement,
            Focus::Files => return,
        };

        match key.code {
            KeyCode::Char(character) => input.insert(character),
            KeyCode::Backspace => input.backspace(),
            KeyCode::Delete => input.delete(),
            KeyCode::Left => input.move_left(),
            KeyCode::Right => input.move_right(),
            KeyCode::Home => input.move_to_start(),
            KeyCode::End => input.move_to_end(),
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

fn shortcut_modifier(modifiers: KeyModifiers) -> bool {
    modifiers.contains(KeyModifiers::CONTROL)
        || (cfg!(target_os = "macos") && modifiers.contains(KeyModifiers::SUPER))
}
