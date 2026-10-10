use thiserror::Error;

#[derive(Error, Debug)]
pub enum LpdfError {
    #[error("Internal Engine Error: {0}")]
    Engine(String),

    #[error("File I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid Input: {0}")]
    InvalidInput(String),

    #[error("Parse Error: {0}")]
    ParseError(String),

    #[error("Configuration Error: {0}")]
    ConfigError(String),

    #[error("Unsupported Operation: {0}")]
    Unsupported(String),
}

pub type Result<T> = std::result::Result<T, LpdfError>;
