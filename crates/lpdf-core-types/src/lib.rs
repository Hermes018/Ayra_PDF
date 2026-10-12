#![forbid(unsafe_code)]

pub mod config;
pub mod error;

pub use config::{
    compute_sha256_hex, compute_sha256_hex_from_reader, inspect_ai_model, AiModelInfo,
    AiModelManifest, AppConfig, AppSettings, GeneralConfig, HardwareTier, OutputPolicy,
    ThemePreference,
};
pub use error::{LpdfError, Result};
