use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::{rename::Rename, selection::Selection, ui::MODIFIER_KEY};

pub struct FileList<'a> {
    entries: &'a Selection<Rename>,
    visible: bool,
}

impl<'a> FileList<'a> {
    pub fn new(entries: &'a Selection<Rename>, visible: bool) -> Self {
        Self { entries, visible }
    }

    pub fn handle_key(entries: &mut Selection<Rename>, key: KeyEvent) {
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
        if !self.visible {
            return;
        }

        let highlighted = self.entries.highlighted();

        let items = self.entries.items().enumerate().map(|(index, entry)| {
            let item_style = if entry.is_selected() {
                Style::default()
            } else {
                Style::default().dim()
            };

            let (marker, marker_style) = if highlighted == Some(index) {
                ("› ", Style::default().fg(Color::Magenta))
            } else if entry.is_selected() {
                ("● ", Style::default().fg(Color::Green))
            } else {
                ("○ ", Style::default().dim())
            };

            let mut line_items = vec![
                Span::styled(marker, marker_style),
                Span::styled(entry.value().source(), item_style),
            ];

            if entry.value().destination() != entry.value().source() {
                line_items.push(Span::styled(" › ", Style::default().fg(Color::Magenta)));
                line_items.push(Span::styled(
                    entry.value().destination(),
                    Style::default().fg(Color::Blue),
                ));
            }

            ListItem::new(Line::from(line_items))
        });

        let list = List::new(items);

        let mut state = ListState::default().with_selected(self.entries.highlighted());

        StatefulWidget::render(list, area, buf, &mut state);
    }
}
