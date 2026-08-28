use crossterm::event::KeyCode;

#[derive(Debug, Default)]
pub struct TextBuffer {
    value: String,
    cursor: usize,
}

impl TextBuffer {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(character) => {
                self.value.insert(self.cursor, character);
                self.cursor += character.len_utf8();
            }
            KeyCode::Backspace if self.cursor > 0 => {
                let previous = self.previous_char_boundary();
                self.value.replace_range(previous..self.cursor, "");
                self.cursor = previous;
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

    fn previous_char_boundary(&self) -> usize {
        self.value[..self.cursor]
            .char_indices()
            .next_back()
            .map(|(index, _)| index)
            .unwrap_or(0)
    }

    fn next_char_boundary(&self) -> usize {
        if self.cursor >= self.value.len() {
            return self.value.len();
        }

        self.value[self.cursor..]
            .char_indices()
            .nth(1)
            .map(|(index, _)| self.cursor + index)
            .unwrap_or(self.value.len())
    }
}
