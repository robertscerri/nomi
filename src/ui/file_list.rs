use std::path::PathBuf;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{List, Widget},
};

use crate::ui::{BlockBuilder, FocusTarget};

#[derive(Debug)]
pub struct FileList {
    directory: PathBuf,
    entries: Vec<String>,
    focused: bool,
}

impl FileList {
    pub fn new(directory: PathBuf, entries: Vec<String>) -> Self {
        FileList {
            directory,
            entries,
            focused: false,
        }
    }
}

impl Widget for &FileList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = BlockBuilder::new()
            .title(format!(" {} ", self.directory.display()))
            .right_title(format!(" {} items ", self.entries.len()))
            .focused(self.is_focused())
            .build();

        List::new(self.entries.iter().map(String::as_str))
            .block(block)
            .render(area, buf);
    }
}

impl FocusTarget for FileList {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn blur(&mut self) {
        self.focused = false;
    }

    fn is_focused(&self) -> bool {
        self.focused
    }
}
