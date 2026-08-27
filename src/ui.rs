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
    let pattern = InputField::new(
        Line::styled(" Pattern ", Style::default().fg(Color::DarkGray)),
        Some(mode_title(app.mode)),
        &app.pattern,
        app.focus == Focus::Pattern,
    );
    let replacement = InputField::new(
        Line::styled(" Replacement ", Style::default().fg(Color::DarkGray)),
        None,
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
    let highlighted = app.focus == Focus::Files && index == app.cursor;
    let directory_suffix = if entry.is_dir { "/" } else { "" };
    let original = entry.name.to_string_lossy();

    let indicator = match (highlighted, entry.selected) {
        (true, _) => Span::styled("› ", Style::default().fg(Color::Magenta)),
        (false, true) => Span::styled("● ", Style::default().fg(Color::White)),
        (false, false) => Span::styled("○ ", Style::default().fg(Color::DarkGray)),
    };
    let original = Span::styled(
        format!("{original}{directory_suffix}"),
        Style::default().fg(if !entry.selected {
            Color::DarkGray
        } else if entry.is_dir {
            Color::Blue
        } else {
            Color::White
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

fn draw_status(frame: &mut Frame, app: &App, preview: &RenamePreview, area: Rect) {
    frame.render_widget(StatusBar::new(app, preview), area);
}

struct InputField<'a> {
    title: Line<'static>,
    right_title: Option<Line<'static>>,
    value: &'a str,
    focused: bool,
}

impl<'a> InputField<'a> {
    fn new(
        title: Line<'static>,
        right_title: Option<Line<'static>>,
        value: &'a str,
        focused: bool,
    ) -> Self {
        Self {
            title,
            right_title,
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
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title_top(self.title.clone())
            .border_style(focus_style(self.focused));
        if let Some(right_title) = &self.right_title {
            block = block.title_top(right_title.clone().right_aligned());
        }
        Paragraph::new(self.value)
            .style(Style::default().fg(Color::White))
            .block(block)
            .render(area, buffer);
    }
}

fn mode_title(mode: MatchMode) -> Line<'static> {
    let (name, color) = match mode {
        MatchMode::Regex => ("Regex", Color::Magenta),
        MatchMode::Literal => ("Literal", Color::Blue),
    };

    Line::from(vec![
        Span::styled(" ● ", Style::default().fg(color)),
        Span::styled(format!("{name} "), Style::default().fg(Color::White)),
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
                Span::styled(message, Style::default().fg(Color::White)),
            ]),
            (None, Some(error)) => Line::from(vec![
                Span::styled("! ", Style::default().fg(Color::Red)),
                Span::styled(error, Style::default().fg(Color::Red)),
            ]),
            (None, None) => Line::from(vec![
                Span::styled("● ", Style::default().fg(Color::Green)),
                Span::styled("Ready", Style::default().fg(Color::White)),
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

        let controls = Line::from(vec![
            keycap("Tab"),
            label(" focus"),
            separator(),
            keycap("Space"),
            label(" select"),
            separator(),
            keycap(mode_shortcut()),
            label(" mode"),
            separator(),
            keycap("Enter"),
            label(" confirm"),
            separator(),
            keycap("Esc"),
            label(" quit"),
        ]);

        Paragraph::new(vec![status, controls])
            .wrap(Wrap { trim: true })
            .render(area, buffer);
    }
}

fn keycap(key: &'static str) -> Span<'static> {
    Span::styled(key, Style::default().fg(Color::Yellow))
}

fn label(label: &'static str) -> Span<'static> {
    Span::styled(label, Style::default().fg(Color::DarkGray))
}

fn separator() -> Span<'static> {
    Span::styled("  │  ", Style::default().fg(Color::DarkGray))
}

#[cfg(target_os = "macos")]
fn mode_shortcut() -> &'static str {
    "Cmd+R"
}

#[cfg(not(target_os = "macos"))]
fn mode_shortcut() -> &'static str {
    "Ctrl+R"
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
        Style::default().fg(Color::DarkGray)
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
