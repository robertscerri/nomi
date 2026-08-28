use std::path::PathBuf;

use clap::{
    Parser,
    builder::{Styles, styling::AnsiColor},
};

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::BrightGreen.on_default().bold())
    .usage(AnsiColor::BrightGreen.on_default().bold())
    .literal(AnsiColor::BrightCyan.on_default().bold())
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Debug, Parser)]
#[command(version, about, styles = STYLES)]
pub struct Cli {
    /// Directory whose immediate children should be renamed
    #[arg(value_name = "DIRECTORY")]
    pub directory: Option<PathBuf>,
}
