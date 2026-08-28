use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("filesystem error: {0}")]
    IO(#[from] std::io::Error),

    #[error("invalid regular expression: {0}")]
    Regex(#[from] regex::Error),

    #[error("{0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, Error>;
