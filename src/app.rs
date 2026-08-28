use std::path::PathBuf;

use crossterm::event::KeyCode;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    widgets::List,
};

use nomi::{
    core::read_entries,
    error::Result,
    ui::{BlockBuilder, FocusTarget, StatusBar, TextInput},
};

#[derive(Debug)]
pub struct App<'a> {
    directory: PathBuf,

    pattern: TextInput,
    replacement: TextInput,
    file_list: List<'a>,
    status_bar: StatusBar,

    exit: bool,
}

impl<'a> App<'a> {
    pub fn try_new(directory: PathBuf) -> Result<Self> {
        let directory = directory.canonicalize()?;
        let entries = read_entries(&directory)?;

        let mut pattern = TextInput::new(" Pattern ");
        pattern.focus();

        let replacement = TextInput::new(" Replacement ");
        let file_list = List::new(entries).block(
            BlockBuilder::new()
                .title(format!(" {} ", directory.display()))
                .right_title(" 5 Items ")
                .build(),
        );
        let status_bar = StatusBar::new();

        Ok(App {
            directory,
            pattern,
            replacement,
            file_list,
            status_bar,
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

        frame.render_widget(&self.file_list, file_list_area);

        frame.render_widget(&self.status_bar, status_area);

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
