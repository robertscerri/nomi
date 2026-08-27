use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
};

use nomi::rename::{Entry, MatchMode, RenamePreview};

use crate::app::{App, Focus};

const INPUT_HEIGHT: u16 = 3;
const STATUS_HEIGHT: u16 = 2;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let [pattern_area, replacement_area, files_area, status_area] = main_layout(area);
    let preview = app.preview();

    draw_inputs(frame, app, pattern_area, replacement_area);
    draw_files(frame, app, &preview, files_area);
    draw_status(frame, app, &preview, status_area);

    if app.confirm {
        draw_confirmation(frame, area, preview.operations.len());
    }
}

fn main_layout(area: Rect) -> [Rect; 4] {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(INPUT_HEIGHT),
            Constraint::Length(INPUT_HEIGHT),
            Constraint::Min(5),
            Constraint::Length(STATUS_HEIGHT),
        ])
        .areas(area)
}

fn draw_inputs(frame: &mut Frame, app: &App, pattern_area: Rect, replacement_area: Rect) {
    let mode = match app.mode {
        MatchMode::Regex => "regex",
        MatchMode::Literal => "literal",
    };

    let pattern = InputField::new(
        format!(" Pattern [{mode}] "),
        &app.pattern,
        app.focus == Focus::Pattern,
    );
    let replacement = InputField::new(
        " Replacement ",
        &app.replacement,
        app.focus == Focus::Replacement,
    );

    frame.render_widget(&pattern, pattern_area);
    frame.render_widget(&replacement, replacement_area);

    if pattern.focused {
        frame.set_cursor_position(pattern.cursor_position(pattern_area));
    } else if replacement.focused {
        frame.set_cursor_position(replacement.cursor_position(replacement_area));
    }
}

fn draw_files(frame: &mut Frame, app: &mut App, preview: &RenamePreview, area: Rect) {
    let visible_rows = area.height.saturating_sub(2) as usize;
    keep_cursor_visible(app, visible_rows);

    let items = app
        .entries
        .iter()
        .enumerate()
        .skip(app.scroll)
        .take(visible_rows)
        .map(|(index, entry)| file_row(app, preview, index, entry));

    let title = format!(
        " {} — {} item(s) ",
        app.directory.display(),
        app.entries.len()
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(focus_style(app.focus == Focus::Files));

    frame.render_widget(List::new(items).block(block), area);
}

fn keep_cursor_visible(app: &mut App, visible_rows: usize) {
    if app.cursor < app.scroll {
        app.scroll = app.cursor;
    } else if visible_rows > 0 && app.cursor >= app.scroll + visible_rows {
        app.scroll = app.cursor + 1 - visible_rows;
    }
}

fn file_row<'a>(
    app: &App,
    preview: &'a RenamePreview,
    index: usize,
    entry: &'a Entry,
) -> ListItem<'a> {
    let marker = if entry.selected { "[x]" } else { "[ ]" };
    let directory_suffix = if entry.is_dir { "/" } else { "" };
    let original = entry.name.to_string_lossy();

    let content = match &preview.names[index] {
        Some(destination) => Line::from(vec![
            Span::raw(format!("{marker} {original}{directory_suffix}")),
            Span::styled("  →  ", Style::default().fg(Color::DarkGray)),
            Span::styled(destination, Style::default().fg(Color::Green)),
        ]),
        None => Line::from(format!("{marker} {original}{directory_suffix}")),
    };

    let highlighted = app.focus == Focus::Files && index == app.cursor;
    ListItem::new(content).style(selection_style(highlighted))
}

fn draw_status(frame: &mut Frame, app: &App, preview: &RenamePreview, area: Rect) {
    frame.render_widget(StatusBar::new(app, preview), area);
}

struct InputField<'a> {
    title: String,
    value: &'a str,
    focused: bool,
}

impl<'a> InputField<'a> {
    fn new(title: impl Into<String>, value: &'a str, focused: bool) -> Self {
        Self {
            title: title.into(),
            value,
            focused,
        }
    }

    fn cursor_position(&self, area: Rect) -> (u16, u16) {
        let text_width = self.value.chars().count() as u16;
        let cursor_x = (area.x + 1 + text_width).min(area.right().saturating_sub(2));
        (cursor_x, area.y + 1)
    }
}

impl Widget for &InputField<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.as_str())
            .border_style(focus_style(self.focused));
        Paragraph::new(self.value).block(block).render(area, buffer);
    }
}

struct StatusBar<'a> {
    message: Option<&'a str>,
    error: Option<&'a str>,
    change_count: usize,
}

impl<'a> StatusBar<'a> {
    fn new(app: &'a App, preview: &'a RenamePreview) -> Self {
        Self {
            message: app.message.as_deref(),
            error: preview.error.as_deref(),
            change_count: preview.len(),
        }
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let status = match (self.message, self.error) {
            (Some(message), _) => Span::styled(message, Style::default().fg(Color::Yellow)),
            (None, Some(error)) => Span::styled(error, Style::default().fg(Color::Red)),
            (None, None) => Span::raw(format!(
                "{} change(s)  •  Tab fields  Space select  Ctrl+R mode  Enter rename  Esc quit",
                self.change_count
            )),
        };

        Paragraph::new(Line::from(status))
            .wrap(Wrap { trim: true })
            .render(area, buffer);
    }
}

fn draw_confirmation(frame: &mut Frame, area: Rect, count: usize) {
    let popup = centered_rect(52, 7, area);
    let content = vec![
        Line::from(""),
        Line::from(format!("Rename {count} item(s)?")),
        Line::from(""),
        Line::from("Enter/Y confirm    N/Esc cancel"),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Confirm ")
        .border_style(Style::default().fg(Color::Yellow));

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(content)
            .alignment(Alignment::Center)
            .block(block),
        popup,
    );
}

fn focus_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    }
}

fn selection_style(selected: bool) -> Style {
    if selected {
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}
