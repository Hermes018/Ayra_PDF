#![forbid(unsafe_code)]

pub mod config;
pub mod error;

pub use config::{AppConfig, GeneralConfig};
pub use error::{LpdfError, Result};
