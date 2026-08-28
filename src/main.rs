mod app;
mod cli;
mod ui;

use std::process::ExitCode;

use clap::Parser;

use nomi::error::NomiError;

use crate::{app::App, cli::Cli};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nomi: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), NomiError> {
    let cli = Cli::parse();
    let directory = cli.directory.unwrap_or(std::env::current_dir()?);
    let mut app = App::new(directory)?;

    let mut terminal = ratatui::try_init()?;
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}
