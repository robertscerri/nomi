use std::{fmt::Display, path::PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
};

use nomi::{core::read_entries, error::Result};

use crate::{
    pluralise,
    ui::{Panel, Stateful, StatusBar, TextBuffer, TextInput, display_path, inner_area},
};

#[cfg(target_os = "macos")]
const MODIFIER_KEY: KeyModifiers = KeyModifiers::SUPER;

#[cfg(not(target_os = "macos"))]
const MODIFIER_KEY: KeyModifiers = KeyModifiers::CONTROL;

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
enum Focus {
    #[default]
    Pattern,
    Replacement,
    FileList,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Pattern => Focus::Replacement,
            Focus::Replacement => Focus::FileList,
            Focus::FileList => Focus::Pattern,
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
enum MatchMode {
    #[default]
    Literal,
    Regex,
}

impl MatchMode {
    pub fn toggle(self) -> Self {
        match self {
            MatchMode::Literal => MatchMode::Regex,
            MatchMode::Regex => MatchMode::Literal,
        }
    }

    pub fn colour(&self) -> Color {
        match self {
            MatchMode::Literal => Color::Blue,
            MatchMode::Regex => Color::Magenta,
        }
    }
}

impl Display for MatchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MatchMode::Literal => "Literal",
            MatchMode::Regex => "Regex",
        })
    }
}

#[derive(Debug)]
struct FileEntry {
    name: String,
    selected: bool,
}

impl FileEntry {
    fn new(name: String) -> Self {
        Self {
            name,
            selected: true,
        }
    }
}

#[derive(Debug)]
pub struct App {
    directory: PathBuf,
    entries: Vec<FileEntry>,
    highlighted_entry: Option<usize>,
    pattern: TextBuffer,
    replacement: TextBuffer,
    focus: Focus,
    match_mode: MatchMode,
    exit: bool,
}

impl App {
    pub fn try_new(directory: PathBuf) -> Result<Self> {
        let directory = directory.canonicalize()?;
        let entries: Vec<_> = read_entries(&directory)?
            .into_iter()
            .map(FileEntry::new)
            .collect();
        let highlighted_entry = (!entries.is_empty()).then_some(0);

        Ok(Self {
            directory,
            entries,
            highlighted_entry,
            pattern: TextBuffer::default(),
            replacement: TextBuffer::default(),
            focus: Focus::default(),
            match_mode: MatchMode::default(),
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

        frame.render_widget(
            Panel::new(
                TextInput::new(self.pattern.value(), self.pattern.cursor()),
                " Pattern ",
            )
            .right_title(Line::from(vec![
                Span::styled(" ● ", self.match_mode.colour()),
                Span::styled(
                    format!("{} ", self.match_mode),
                    Style::default().fg(Color::Reset),
                ),
            ]))
            .focused(self.focus == Focus::Pattern),
            pattern_area,
        );

        frame.render_widget(
            Panel::new(
                TextInput::new(self.replacement.value(), self.replacement.cursor()),
                " Replacement ",
            )
            .focused(self.focus == Focus::Replacement),
            replacement_area,
        );

        let file_items = self.entries.iter().map(|entry| {
            let marker = if entry.selected { "[x]" } else { "[ ]" };
            let marker_style = if entry.selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().dim()
            };

            ListItem::new(Line::from(vec![
                Span::styled(marker, marker_style),
                Span::raw(format!(" {}", entry.name)),
            ]))
        });

        let file_list = List::new(file_items)
            .highlight_symbol("› ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let file_list_state = ListState::default().with_selected(self.highlighted_entry);

        frame.render_widget(
            Panel::new(
                Stateful::new(file_list, file_list_state),
                format!(" {} ", display_path(self.directory.as_path())),
            )
            .right_title(format!(
                " {} ",
                pluralise!(self.entries.len(), "item", "items")
            ))
            .focused(self.focus == Focus::FileList),
            file_list_area,
        );

        frame.render_widget(StatusBar::new(), status_area);

        match self.focus {
            Focus::Pattern => self.set_text_cursor(frame, pattern_area, &self.pattern),
            Focus::Replacement => self.set_text_cursor(frame, replacement_area, &self.replacement),
            Focus::FileList => {}
        }
    }

    fn set_text_cursor(&self, frame: &mut Frame, area: Rect, buffer: &TextBuffer) {
        let input = TextInput::new(buffer.value(), buffer.cursor());
        frame.set_cursor_position(input.cursor_position(inner_area(area)));
    }

    fn handle_events(&mut self) -> Result<()> {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()?
            && key.is_press()
        {
            self.handle_key(key);
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(MODIFIER_KEY) {
            match key.code {
                KeyCode::Char('r') => self.match_mode = self.match_mode.toggle(),
                _ => self.handle_focused_key(key),
            }
        } else {
            match key.code {
                KeyCode::Esc => self.exit = true,
                KeyCode::Tab => self.focus = self.focus.next(),
                _ => self.handle_focused_key(key),
            }
        }
    }

    fn handle_focused_key(&mut self, key: KeyEvent) {
        match self.focus {
            Focus::Pattern => self.pattern.handle_key(key),
            Focus::Replacement => self.replacement.handle_key(key),
            Focus::FileList => self.handle_file_list_key(key),
        }
    }

    fn handle_file_list_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.highlight_previous_entry(),
            KeyCode::Down => self.highlight_next_entry(),
            KeyCode::Char(' ') => self.toggle_highlighted_entry(),
            KeyCode::Char('a') if key.modifiers.contains(MODIFIER_KEY) => self.toggle_all_entries(),
            _ => {}
        }
    }

    fn highlight_previous_entry(&mut self) {
        if let Some(highlighted) = self.highlighted_entry.as_mut() {
            *highlighted = highlighted.saturating_sub(1);
        }
    }

    fn highlight_next_entry(&mut self) {
        if let Some(highlighted) = self.highlighted_entry.as_mut() {
            *highlighted = highlighted
                .saturating_add(1)
                .min(self.entries.len().saturating_sub(1));
        }
    }

    fn toggle_highlighted_entry(&mut self) {
        if let Some(entry) = self
            .highlighted_entry
            .and_then(|highlighted| self.entries.get_mut(highlighted))
        {
            entry.selected = !entry.selected;
        }
    }

    fn toggle_all_entries(&mut self) {
        let selected = !self.entries.iter().all(|entry| entry.selected);

        for entry in &mut self.entries {
            entry.selected = selected;
        }
    }
}
