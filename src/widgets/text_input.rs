use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

#[derive(Debug, Default)]
pub struct InputState {
    value: String,
    cursor: usize,
}

impl InputState {
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

            KeyCode::Left => {
                self.cursor = self.previous_char_boundary();
            }

            KeyCode::Right => {
                self.cursor = self.next_char_boundary();
            }

            // TODO: Add Delete, Home, End
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

pub struct TextInput<'a> {
    title: &'a str,
}

impl<'a> TextInput<'a> {
    pub fn new(title: &'a str) -> Self {
        TextInput { title }
    }
}

impl StatefulWidget for TextInput<'_> {
    type State = InputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default().title(self.title).borders(Borders::ALL);

        let inner = block.inner(area);

        block.render(area, buf);

        Paragraph::new(state.value()).render(inner, buf);
    }
}
