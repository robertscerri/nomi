use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug)]
pub struct TextInput {
    title: &'static str,
    value: String,
    cursor: usize,
}

impl TextInput {
    pub fn new(title: &'static str) -> Self {
        Self {
            title,
            value: String::new(),
            cursor: 0,
        }
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

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().dim()
        };

        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(style);

        let inner = block.inner(area);
        let cursor_column = Line::raw(&self.value[..self.cursor]).width();
        let scroll = cursor_column.saturating_sub(inner.width.saturating_sub(1) as usize);

        frame.render_widget(
            Paragraph::new(self.value.as_str())
                .block(block)
                .scroll((0, scroll as u16)),
            area,
        );

        if focused {
            let cursor_position = inner.x + (cursor_column - scroll) as u16;
            frame.set_cursor_position((cursor_position, inner.y));
        }
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
