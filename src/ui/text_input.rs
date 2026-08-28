use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::ui::BlockBuilder;

#[derive(Debug)]
pub struct TextInput {
    title: &'static str,
    value: String,
    cursor: usize,
    focused: bool,
}

impl TextInput {
    pub fn new(title: &'static str) -> Self {
        Self {
            title,
            value: String::new(),
            cursor: 0,
            focused: false,
        }
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn blur(&mut self) {
        self.focused = false;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }

            KeyCode::Backspace => {
                if self.cursor > 0 {
                    let previous = self.previous_char_boundary();
                    self.value.replace_range(previous..self.cursor, "");
                    self.cursor = previous;
                }
            }
            KeyCode::Delete if self.cursor < self.value.len() => {
                self.value
                    .replace_range(self.cursor..self.next_char_boundary(), "");
            }
            KeyCode::Left => self.cursor = self.previous_char_boundary(),
            KeyCode::Right => self.cursor = self.next_char_boundary(),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.value.len(),
            _ => {}
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    pub fn cursor_position(&self, area: Rect) -> (u16, u16) {
        let (inner, cursor_column, scroll) = self.viewport(area);

        (inner.x + (cursor_column - scroll) as u16, inner.y)
    }

    fn viewport(&self, area: Rect) -> (Rect, usize, usize) {
        let inner = Block::bordered().inner(area);
        let cursor_column = Line::raw(&self.value[..self.cursor]).width();
        let scroll = cursor_column.saturating_sub(inner.width.saturating_sub(1) as usize);

        (inner, cursor_column, scroll)
    }

    fn previous_char_boundary(&self) -> usize {
        self.value[..self.cursor]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn next_char_boundary(&self) -> usize {
        if self.cursor >= self.value.len() {
            return self.value.len();
        }

        self.value[self.cursor..]
            .char_indices()
            .nth(1)
            .map(|(i, _)| self.cursor + i)
            .unwrap_or(self.value.len())
    }
}

impl Widget for &TextInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = BlockBuilder::new()
            .title(self.title)
            .focused(self.focused)
            .build();

        let (_, _, scroll) = self.viewport(area);

        Paragraph::new(self.value.as_str())
            .block(block)
            .scroll((0, scroll as u16))
            .render(area, buf);
    }
}
