use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use nomi::rename::{Entry, MatchMode, Preview};

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

    draw_input(
        frame,
        pattern_area,
        &format!(" Pattern [{mode}] "),
        &app.pattern,
        app.focus == Focus::Pattern,
    );
    draw_input(
        frame,
        replacement_area,
        " Replacement ",
        &app.replacement,
        app.focus == Focus::Replacement,
    );
}

fn draw_files(frame: &mut Frame, app: &mut App, preview: &Preview, area: Rect) {
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

fn file_row<'a>(app: &App, preview: &'a Preview, index: usize, entry: &'a Entry) -> ListItem<'a> {
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

fn draw_status(frame: &mut Frame, app: &App, preview: &Preview, area: Rect) {
    let status = match (&app.message, &preview.error) {
        (Some(message), _) => Span::styled(message, Style::default().fg(Color::Yellow)),
        (None, Some(error)) => Span::styled(error, Style::default().fg(Color::Red)),
        (None, None) => Span::raw(format!(
            "{} change(s)  •  Tab fields  Space select  Ctrl+R mode  Enter rename  Esc quit",
            preview.operations.len()
        )),
    };

    frame.render_widget(
        Paragraph::new(Line::from(status)).wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_input(frame: &mut Frame, area: Rect, title: &str, value: &str, focused: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(focus_style(focused));
    frame.render_widget(Paragraph::new(value).block(block), area);

    if focused {
        let text_width = value.chars().count() as u16;
        let cursor_x = (area.x + 1 + text_width).min(area.right().saturating_sub(2));
        frame.set_cursor_position((cursor_x, area.y + 1));
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
