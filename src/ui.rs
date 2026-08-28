mod status_bar;
mod text_input;

use ratatui::style::{Color, Style};

pub use status_bar::StatusBar;
pub use text_input::TextInput;

pub fn focus_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().dim()
    }
}
