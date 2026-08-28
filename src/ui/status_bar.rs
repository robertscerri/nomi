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
        let [status_area, controls_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);

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

        Paragraph::new(Line::from(vec![])).render(status_area, buf);
        Paragraph::new(controls).render(controls_area, buf);
    }
}
