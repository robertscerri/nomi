use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{List, Widget},
};

// TODO: Do we need FileList?
#[derive(Debug)]
pub struct FileList<'a> {
    entries: &'a [String],
}

impl<'a> FileList<'a> {
    pub fn new(entries: &'a [String]) -> Self {
        Self { entries }
    }
}

impl Widget for FileList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        List::new(self.entries.iter().map(String::as_str)).render(area, buf);
    }
}
