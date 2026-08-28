mod app;
mod cli;
mod ui;

use std::process::ExitCode;

use clap::Parser;
use nomi::error::Result;

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

fn run() -> Result<()> {
    let cli = Cli::parse();
    let directory = cli.directory.unwrap_or(std::env::current_dir()?);

    ratatui::run(|terminal| App::try_new(directory)?.run(terminal))?;

    Ok(())
}
