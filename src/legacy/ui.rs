use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
};

use nomi::rename::{Entry, MatchMode, RenamePreview};

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

fn draw_files(frame: &mut Frame, app: &App, preview: &RenamePreview, area: Rect) {
    let visible_rows = area.height.saturating_sub(2) as usize;
    let scroll = app.cursor.saturating_add(1).saturating_sub(visible_rows);

    let items = app
        .entries
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_rows)
        .map(|(index, entry)| file_row(app, preview, index, entry));

    let item_label = if app.entries.len() == 1 {
        "item"
    } else {
        "items"
    };
    let path_title = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            display_path(&app.directory),
            Style::default().fg(Color::Blue),
        ),
        Span::styled(" ", Style::default()),
    ]);
    let count_title = Line::styled(
        format!(" {} {item_label} ", app.entries.len()),
        Style::default().fg(Color::DarkGray),
    )
    .right_aligned();
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(path_title)
        .title_top(count_title)
        .border_style(focus_style(app.focus == Focus::Files));

    frame.render_widget(List::new(items).block(block), area);
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

fn mode_title(mode: MatchMode) -> Line<'static> {
    let (name, color) = match mode {
        MatchMode::Regex => ("Regex", Color::Magenta),
        MatchMode::Literal => ("Literal", Color::Blue),
    };

    Line::from(vec![
        Span::styled(" ● ", Style::default().fg(color)),
        Span::styled(format!("{name} "), Style::default().fg(Color::Reset)),
    ])
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
            (Some(message), _) => Line::from(vec![
                Span::styled("● ", Style::default().fg(Color::Blue)),
                Span::raw(message),
            ]),
            (None, Some(error)) => Line::from(vec![
                Span::styled("! ", Style::default().fg(Color::Red)),
                Span::styled(error, Style::default().fg(Color::Red)),
            ]),
            (None, None) => Line::from(vec![
                Span::styled("● ", Style::default().fg(Color::Green)),
                Span::raw("Ready"),
                Span::styled(" — ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} pending change(s)", self.change_count),
                    Style::default().fg(if self.change_count == 0 {
                        Color::DarkGray
                    } else {
                        Color::Green
                    }),
                ),
            ]),
        };

        let [status_area, controls_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
        let identity = Line::from(vec![
            Span::styled("nomi ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                concat!("v", env!("CARGO_PKG_VERSION")),
                Style::default().fg(Color::Blue),
            ),
        ]);

        let identity_fits = status.width() + identity.width() + 2 <= status_area.width as usize;
        Paragraph::new(status).render(status_area, buffer);
        if identity_fits {
            Paragraph::new(identity)
                .alignment(Alignment::Right)
                .render(status_area, buffer);
        }
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

fn display_path(path: &std::path::Path) -> String {
    let displayed = path.display().to_string();

    if let Some(unc_path) = displayed.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc_path}")
    } else if let Some(local_path) = displayed.strip_prefix(r"\\?\") {
        local_path.to_owned()
    } else {
        displayed
    }
}
