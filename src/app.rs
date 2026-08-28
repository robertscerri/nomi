use std::path::PathBuf;

use crossterm::event::KeyCode;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    widgets::Block,
};

use nomi::{error::Result, ui::TextInput};

#[derive(Debug)]
pub struct App {
    directory: PathBuf,

    pattern: TextInput,
    replacement: TextInput,

    exit: bool,
}

impl App {
    pub fn try_new(directory: PathBuf) -> Result<Self> {
        let directory = directory.canonicalize()?;

        let mut pattern = TextInput::new(" Pattern ");
        let replacement = TextInput::new(" Replacement ");

        pattern.focus();

        Ok(App {
            directory,
            pattern,
            replacement,
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

        frame.render_widget(&self.pattern, pattern_area);
        frame.render_widget(&self.replacement, replacement_area);

        frame.render_widget(
            Block::bordered().title(format!(" {} ", self.directory.display())),
            file_list_area,
        );
        frame.render_widget(Block::new(), status_area);

        if self.pattern.is_focused() {
            frame.set_cursor_position(self.pattern.cursor_position(pattern_area));
        } else if self.replacement.is_focused() {
            frame.set_cursor_position(self.replacement.cursor_position(replacement_area));
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            // Only handle event when key is pressed down
            if !key.is_press() {
                return Ok(());
            }

            match key.code {
                KeyCode::Esc => self.exit = true,
                KeyCode::Tab => self.change_focus(),

                _ if self.pattern.is_focused() => self.pattern.handle_key(key),
                _ if self.replacement.is_focused() => self.replacement.handle_key(key),
                _ => {}
            }
        }

        Ok(())
    }

    fn change_focus(&mut self) {
        if self.pattern.is_focused() {
            self.pattern.blur();
            self.replacement.focus();
        } else if self.replacement.is_focused() {
            self.replacement.blur();
            //TODO self.file_list.focus();
        } else {
            //TODO self.file_list.blur();
            self.pattern.focus();
        }
    }
}
