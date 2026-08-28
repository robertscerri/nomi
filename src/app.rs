use std::path::PathBuf;

use crossterm::event::KeyCode;
use nomi::{error::Result, widgets::TextInput};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    widgets::Block,
};

#[derive(Debug, Default)]
pub enum FocusedField {
    #[default]
    Pattern,
    Replacement,
}

#[derive(Debug)]
pub struct App {
    directory: PathBuf,

    pattern: TextInput,
    replacement: TextInput,

    focused: FocusedField,

    exit: bool,
}

impl App {
    pub fn try_new(directory: PathBuf) -> Result<Self> {
        let directory = directory.canonicalize()?;

        Ok(App {
            directory,
            pattern: TextInput::new(" Pattern "),
            replacement: TextInput::new(" Replacement "),
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

    fn draw(&self, frame: &mut Frame) {
        let [pattern_area, replacement_area, file_list_area, status_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(frame.area());

        self.pattern.render(
            frame,
            pattern_area,
            matches!(self.focused, FocusedField::Pattern),
        );

        self.replacement.render(
            frame,
            replacement_area,
            matches!(self.focused, FocusedField::Replacement),
        );

        frame.render_widget(
            Block::bordered().title(format!(" {} ", self.directory.display())),
            file_list_area,
        );
        frame.render_widget(Block::new(), status_area);
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
