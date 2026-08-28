use std::path::PathBuf;

use crossterm::event::KeyCode;
use nomi::{
    error::Result,
    widgets::{InputState, TextInput},
};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, StatefulWidget, Widget},
};

#[derive(Debug, Default)]
pub enum FocusedField {
    #[default]
    Pattern,
    Replacement,
}

#[derive(Debug, Default)]
pub struct App {
    directory: PathBuf,

    pattern: InputState,
    replacement: InputState,

    focused: FocusedField,

    exit: bool,
}

impl App {
    pub fn try_new(directory: PathBuf) -> Result<Self> {
        let directory = directory.canonicalize()?;

        Ok(App {
            directory,
            pattern: InputState::default(),
            replacement: InputState::default(),
            focused: FocusedField::Pattern,
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

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            match key.code {
                KeyCode::Esc => {
                    self.exit = true;
                }

                KeyCode::Tab => {
                    self.focused = match self.focused {
                        FocusedField::Pattern => FocusedField::Replacement,
                        FocusedField::Replacement => FocusedField::Pattern,
                    }
                }

                _ => match self.focused {
                    FocusedField::Pattern => self.pattern.handle_key(key),
                    FocusedField::Replacement => self.replacement.handle_key(key),
                },
            }
        }

        Ok(())
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [pattern_area, replacement_area, file_list_area, status_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(area);

        TextInput::new(" Pattern ").render(pattern_area, buf, &mut self.pattern);
        TextInput::new(" Replacement ").render(replacement_area, buf, &mut self.replacement);

        Block::new()
            .title(" Files ")
            .borders(ratatui::widgets::Borders::ALL)
            .render(file_list_area, buf);
        Block::new().render(status_area, buf);
    }
}
