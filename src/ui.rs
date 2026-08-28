mod panel;
mod status_bar;
mod text_buffer;
mod text_input;

pub use panel::{Panel, inner_area};
pub use status_bar::StatusBar;
pub use text_buffer::TextBuffer;
pub use text_input::TextInput;

pub fn display_path(path: &std::path::Path) -> String {
    let displayed = path.display().to_string();

    if let Some(unc_path) = displayed.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc_path}")
    } else if let Some(local_path) = displayed.strip_prefix(r"\\?\") {
        local_path.to_owned()
    } else {
        displayed
    }
}

#[macro_export]
macro_rules! pluralise {
    ($count:expr, $singular:literal, $plural:literal) => {{
        let count = $count;
        format!("{} {}", count, if count == 1 { $singular } else { $plural })
    }};
}
