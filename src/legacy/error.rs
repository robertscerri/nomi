use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NomiError {
    #[error("filesystem error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid regular expression: {0}")]
    Regex(#[from] regex::Error),

    #[error("{0}")]
    Validation(String),

    #[error("rename from '{}' to '{}' failed: {source}", .from.display(), .to.display())]
    Rename {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("rename failed and rollback was incomplete: {message}")]
    Rollback { message: String },
}
