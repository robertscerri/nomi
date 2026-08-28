use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    widgets::List,
};

use nomi::{core::read_entries, error::Result};

use crate::ui::{Panel, StatusBar, TextInput, inner_area};

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
    pattern: String,
    pattern_cursor: usize,
    replacement: String,
    replacement_cursor: usize,
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
            pattern: String::new(),
            pattern_cursor: 0,
            replacement: String::new(),
            replacement_cursor: 0,
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
                TextInput::new(&self.pattern, self.pattern_cursor),
                " Pattern ",
            )
            .focused(self.focus == Focus::Pattern),
            pattern_area,
        );

        frame.render_widget(
            Panel::new(
                TextInput::new(&self.replacement, self.replacement_cursor),
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
            Focus::Pattern => {
                self.set_text_cursor(frame, pattern_area, &self.pattern, self.pattern_cursor)
            }
            Focus::Replacement => self.set_text_cursor(
                frame,
                replacement_area,
                &self.replacement,
                self.replacement_cursor,
            ),
            Focus::FileList => {}
        }
    }

    fn set_text_cursor(&self, frame: &mut Frame, area: Rect, value: &str, cursor: usize) {
        let input = TextInput::new(value, cursor);
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
            _ => self.edit_focused_input(key),
        }
    }

    // TODO: The tuple syntax is a bit meh
    fn edit_focused_input(&mut self, key: KeyEvent) {
        let (value, cursor) = match self.focus {
            Focus::Pattern => (&mut self.pattern, &mut self.pattern_cursor),
            Focus::Replacement => (&mut self.replacement, &mut self.replacement_cursor),
            Focus::FileList => return,
        };

        match key.code {
            KeyCode::Char(character) => {
                value.insert(*cursor, character);
                *cursor += character.len_utf8();
            }
            KeyCode::Backspace if *cursor > 0 => {
                let previous = previous_char_boundary(value, *cursor);
                value.replace_range(previous..*cursor, "");
                *cursor = previous;
            }
            KeyCode::Delete if *cursor < value.len() => {
                value.replace_range(*cursor..next_char_boundary(value, *cursor), "");
            }
            KeyCode::Left => *cursor = previous_char_boundary(value, *cursor),
            KeyCode::Right => *cursor = next_char_boundary(value, *cursor),
            KeyCode::Home => *cursor = 0,
            KeyCode::End => *cursor = value.len(),
            _ => {}
        }
    }
}

// TODO: Cursor text logic can probably be abstracted away into some encapsulation together with { .value, .cursor }
fn previous_char_boundary(value: &str, cursor: usize) -> usize {
    value[..cursor]
        .char_indices()
        .next_back()
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn next_char_boundary(value: &str, cursor: usize) -> usize {
    if cursor >= value.len() {
        return value.len();
    }

    value[cursor..]
        .char_indices()
        .nth(1)
        .map(|(index, _)| cursor + index)
        .unwrap_or(value.len())
}
