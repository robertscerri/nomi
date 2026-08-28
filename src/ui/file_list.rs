use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::{selection::Selection, ui::MODIFIER_KEY};

pub struct FileList<'a> {
    entries: &'a Selection<String>,
}

impl<'a> FileList<'a> {
    pub fn new(entries: &'a Selection<String>) -> Self {
        Self { entries }
    }

    pub fn handle_key(entries: &mut Selection<String>, key: KeyEvent) {
        match key.code {
            KeyCode::Up => entries.highlight_previous(),
            KeyCode::Down => entries.highlight_next(),
            KeyCode::Char(' ') => entries.toggle_highlighted(),
            KeyCode::Char('a') if key.modifiers.contains(MODIFIER_KEY) => entries.toggle_all(),
            _ => {}
        }
    }
}

impl Widget for FileList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items = self.entries.items().map(|entry| {
            let marker = if entry.is_selected() { "[x]" } else { "[ ]" };
            let marker_style = if entry.is_selected() {
                Style::default().fg(Color::Green)
            } else {
                Style::default().dim()
            };

            ListItem::new(Line::from(vec![
                Span::styled(marker, marker_style),
                Span::raw(format!(" {}", entry.value())),
            ]))
        });

        let list = List::new(items)
            .highlight_symbol("› ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let mut state = ListState::default().with_selected(self.entries.highlighted());

        StatefulWidget::render(list, area, buf, &mut state);
    }
}
