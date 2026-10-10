use crate::error::{LpdfError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub default_output_dir: Option<String>,
    pub hw_tier_override: Option<String>,
    pub theme: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            general: GeneralConfig {
                default_output_dir: None,
                hw_tier_override: None,
                theme: Some("system".to_string()),
            },
        }
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(LpdfError::Io)?;
        let config: AppConfig =
            toml::from_str(&content).map_err(|e| LpdfError::ConfigError(e.to_string()))?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content =
            toml::to_string_pretty(self).map_err(|e| LpdfError::ConfigError(e.to_string()))?;
        fs::write(path, content).map_err(LpdfError::Io)?;
        Ok(())
    }
}
