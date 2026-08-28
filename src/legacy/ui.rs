use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
};

use nomi::rename::{Entry, RenamePreview};

use crate::app::{App, Focus};

const INPUT_HEIGHT: u16 = 3;
const STATUS_HEIGHT: u16 = 2;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let [pattern_area, replacement_area, files_area, status_area] = main_layout(area);
    let preview = app.preview();

    draw_inputs(frame, app, pattern_area, replacement_area);
    draw_files(frame, app, &preview, files_area);
    frame.render_widget(StatusBar::new(app, &preview), status_area);

    if app.confirm {
        draw_confirmation(frame, area, preview.operations.len());
    }
}

fn file_row<'a>(
    app: &App,
    preview: &'a RenamePreview,
    index: usize,
    entry: &'a Entry,
) -> ListItem<'a> {
    let highlighted = app.focus == Focus::Files && index == app.cursor;
    let directory_suffix = if entry.is_dir { "/" } else { "" };
    let original = entry.name.to_string_lossy();

    let indicator = match (highlighted, entry.selected) {
        (true, _) => Span::styled("› ", Style::default().fg(Color::Magenta)),
        (false, true) => Span::raw("● "),
        (false, false) => Span::styled("○ ", Style::default().fg(Color::DarkGray)),
    };
    let original = Span::styled(
        format!("{original}{directory_suffix}"),
        Style::default().fg(if !entry.selected {
            Color::DarkGray
        } else if entry.is_dir {
            Color::Blue
        } else {
            Color::Reset
        }),
    );

    let content = match &preview.names[index] {
        Some(destination) => Line::from(vec![
            indicator,
            original,
            Span::styled("  ›  ", Style::default().fg(Color::Magenta)),
            Span::styled(destination, Style::default().fg(Color::Green)),
        ]),
        None => Line::from(vec![indicator, original]),
    };

    ListItem::new(content)
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
