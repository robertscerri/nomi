use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Directory whose immediate children should be renamed
    #[arg(value_name = "DIRECTORY")]
    pub directory: Option<PathBuf>,
}
