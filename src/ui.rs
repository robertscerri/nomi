use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::{
    app::{App, Focus},
    rename::MatchMode,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(area);

    let mode = match app.mode {
        MatchMode::Regex => "regex",
        MatchMode::Literal => "literal",
    };
    draw_input(
        frame,
        chunks[0],
        &format!(" Pattern [{mode}] "),
        &app.pattern,
        app.focus == Focus::Pattern,
    );
    draw_input(
        frame,
        chunks[1],
        " Replacement ",
        &app.replacement,
        app.focus == Focus::Replacement,
    );

    let preview = app.preview();
    let visible_height = chunks[2].height.saturating_sub(2) as usize;
    if app.cursor < app.scroll {
        app.scroll = app.cursor;
    } else if visible_height > 0 && app.cursor >= app.scroll + visible_height {
        app.scroll = app.cursor + 1 - visible_height;
    }

    let items = app
        .entries
        .iter()
        .enumerate()
        .skip(app.scroll)
        .take(visible_height)
        .map(|(index, entry)| {
            let marker = if entry.selected { "[x]" } else { "[ ]" };
            let kind = if entry.is_dir { "/" } else { "" };
            let original = entry.name.to_string_lossy();
            let line = if let Some(destination) = &preview.names[index] {
                Line::from(vec![
                    Span::raw(format!("{marker} {original}{kind}")),
                    Span::styled("  →  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(destination, Style::default().fg(Color::Green)),
                ])
            } else {
                Line::from(format!("{marker} {original}{kind}"))
            };
            let style = if app.focus == Focus::Files && index == app.cursor {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect::<Vec<_>>();

    let title = format!(
        " {} — {} item(s) ",
        app.directory.display(),
        app.entries.len()
    );
    frame.render_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(focus_style(app.focus == Focus::Files)),
        ),
        chunks[2],
    );

    let status = if let Some(message) = &app.message {
        Span::styled(message, Style::default().fg(Color::Yellow))
    } else if let Some(error) = &preview.error {
        Span::styled(error, Style::default().fg(Color::Red))
    } else {
        Span::raw(format!(
            "{} change(s)  •  Tab fields  Space select  Ctrl+R mode  Enter rename  Esc quit",
            preview.operations.len()
        ))
    };
    frame.render_widget(
        Paragraph::new(Line::from(status)).wrap(Wrap { trim: true }),
        chunks[3],
    );

    if app.confirm {
        draw_confirmation(frame, area, preview.operations.len());
    }
}

fn draw_input(frame: &mut Frame, area: Rect, title: &str, value: &str, focused: bool) {
    frame.render_widget(
        Paragraph::new(value).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(focus_style(focused)),
        ),
        area,
    );
    if focused {
        let cursor_x = area.x + 1 + value.chars().count() as u16;
        frame.set_cursor_position((cursor_x.min(area.right().saturating_sub(2)), area.y + 1));
    }
}

fn draw_confirmation(frame: &mut Frame, area: Rect, count: usize) {
    let popup = centered_rect(52, 7, area);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(format!("Rename {count} item(s)?")),
            Line::from(""),
            Line::from("Enter/Y confirm    N/Esc cancel"),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirm ")
                .border_style(Style::default().fg(Color::Yellow)),
        ),
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
