use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

macro_rules! key {
    ($key:literal) => {
        Span::styled($key, Style::default().fg(Color::Yellow))
    };
}

macro_rules! label {
    ($label:literal) => {
        Span::styled(concat!(" ", $label), Style::default().fg(Color::DarkGray))
    };
}

macro_rules! separator {
    () => {
        Span::styled("  |  ", Style::default().fg(Color::DarkGray))
    };
}

#[derive(Debug, Default)]
pub struct StatusBar {}

impl StatusBar {
    pub fn new() -> Self {
        StatusBar {}
    }
}

impl Widget for StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let branding = Line::from(vec![
            Span::styled(
                format!("{} ", env!("CARGO_PKG_NAME")),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("v{}", env!("CARGO_PKG_VERSION")),
                Style::default().fg(Color::Blue),
            ),
        ]);

        let branding_width = branding.width() as u16;

        let [controls_area, branding_area] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(branding_width)])
                .areas(area);

        let controls = Line::from(vec![
            key!("Tab"),
            label!("focus"),
            separator!(),
            key!("Space"),
            label!("select"),
            separator!(),
            key!("Ctrl+R"),
            label!("mode"),
            separator!(),
            key!("Enter"),
            label!("confirm"),
            separator!(),
            key!("Esc"),
            label!("quit"),
        ]);

        Paragraph::new(controls).render(controls_area, buf);
        Paragraph::new(branding)
            .right_aligned()
            .render(branding_area, buf);
    }
}
