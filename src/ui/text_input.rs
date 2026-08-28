use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Paragraph, Widget},
};

#[derive(Debug)]
pub struct TextInput<'a> {
    value: &'a str,
    cursor: usize,
}

impl<'a> TextInput<'a> {
    pub fn new(value: &'a str, cursor: usize) -> Self {
        debug_assert!(value.is_char_boundary(cursor));
        Self { value, cursor }
    }

    pub fn cursor_position(&self, area: Rect) -> (u16, u16) {
        let (cursor_column, scroll) = self.viewport(area);

        (area.x + (cursor_column - scroll) as u16, area.y)
    }

    fn viewport(&self, area: Rect) -> (usize, usize) {
        let cursor_column = Line::raw(&self.value[..self.cursor]).width();
        let scroll = cursor_column.saturating_sub(area.width.saturating_sub(1) as usize);

        (cursor_column, scroll)
    }
}

impl Widget for TextInput<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (_, scroll) = self.viewport(area);

        Paragraph::new(self.value)
            .scroll((0, scroll as u16))
            .render(area, buf);
    }
}
