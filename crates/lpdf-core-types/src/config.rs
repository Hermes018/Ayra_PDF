use crate::error::{LpdfError, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum HardwareTier {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum OutputPolicy {
    #[default]
    SuffixNewFile,
    Subfolder,
    OverwriteWithBackup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct AppSettings {
    pub tier_override: Option<HardwareTier>,
    pub output_policy: OutputPolicy,
    pub backups_enabled: bool,
    pub language: String,
    pub theme: ThemePreference,
    pub local_ai_model_path: Option<PathBuf>,
    pub ai_local_enabled: bool,
    #[serde(default)]
    pub ai_license_acknowledged: bool,
    pub ai_model_sha256: Option<String>,
    pub ai_model_license: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            tier_override: None,
            output_policy: OutputPolicy::SuffixNewFile,
            backups_enabled: true,
            language: "en".to_string(),
            theme: ThemePreference::System,
            local_ai_model_path: None,
            ai_local_enabled: false,
            ai_license_acknowledged: false,
            ai_model_sha256: None,
            ai_model_license: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct AiModelInfo {
    pub path: PathBuf,
    pub file_name: String,
    pub size_bytes: u64,
    pub sha256_hex: String,
    pub format: String,
    #[serde(default)]
    pub license_spdx: Option<String>,
    #[serde(default)]
    pub license_acknowledged: bool,
}

/// Manifest describing a verified local AI model with licensing terms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct AiModelManifest {
    pub model_path: PathBuf,
    pub sha256_hex: String,
    pub format: String,
    pub license_spdx: String,
    pub license_acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    pub general: GeneralConfig,
    pub settings: Option<AppSettings>,
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
            settings: Some(AppSettings::default()),
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

/// Compute SHA-256 hash in hexadecimal from a streaming reader without buffering entire file in memory.
pub fn compute_sha256_hex_from_reader<R: Read>(mut reader: R) -> std::io::Result<String> {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let mut buffer = [0u8; 64];
    let mut buf_len = 0usize;
    let mut total_bytes = 0u64;

    let mut chunk = [0u8; 8192];
    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        total_bytes += n as u64;
        let mut offset = 0;
        while offset < n {
            let to_copy = (64 - buf_len).min(n - offset);
            buffer[buf_len..buf_len + to_copy].copy_from_slice(&chunk[offset..offset + to_copy]);
            buf_len += to_copy;
            offset += to_copy;

            if buf_len == 64 {
                process_block(&buffer, &mut h, &K);
                buf_len = 0;
            }
        }
    }

    // Padding
    buffer[buf_len] = 0x80;
    buf_len += 1;

    if buf_len > 56 {
        buffer[buf_len..64].fill(0);
        process_block(&buffer, &mut h, &K);
        buf_len = 0;
    }

    buffer[buf_len..56].fill(0);
    let total_bits = total_bytes.wrapping_mul(8);
    buffer[56..64].copy_from_slice(&total_bits.to_be_bytes());
    process_block(&buffer, &mut h, &K);

    let mut hex = String::with_capacity(64);
    for word in h {
        hex.push_str(&format!("{:08x}", word));
    }
    Ok(hex)
}

fn process_block(block: &[u8; 64], h: &mut [u32; 8], k: &[u32; 64]) {
    let mut w = [0u32; 64];
    for i in 0..16 {
        w[i] = u32::from_be_bytes([
            block[i * 4],
            block[i * 4 + 1],
            block[i * 4 + 2],
            block[i * 4 + 3],
        ]);
    }
    for i in 16..64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
    }

    let mut a = h[0];
    let mut b = h[1];
    let mut c = h[2];
    let mut d = h[3];
    let mut e = h[4];
    let mut f = h[5];
    let mut g = h[6];
    let mut mut_h = h[7];

    for i in 0..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ ((!e) & g);
        let temp1 = mut_h
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(k[i])
            .wrapping_add(w[i]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);

        mut_h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c);
    h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e);
    h[5] = h[5].wrapping_add(f);
    h[6] = h[6].wrapping_add(g);
    h[7] = h[7].wrapping_add(mut_h);
}

/// Computes SHA-256 hex digest for a file on disk.
pub fn compute_sha256_hex(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(LpdfError::Io)?;
    compute_sha256_hex_from_reader(file).map_err(LpdfError::Io)
}

/// Inspects and validates a local AI model file, extracting format, file size, and SHA-256 digest.
pub fn inspect_ai_model(path: &Path) -> Result<AiModelInfo> {
    if !path.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Model file does not exist: {}",
            path.display()
        )));
    }

    let meta = fs::metadata(path).map_err(LpdfError::Io)?;
    if meta.is_dir() {
        return Err(LpdfError::InvalidInput(
            "Model path is a directory, expected a file".to_string(),
        ));
    }

    let size_bytes = meta.len();
    if size_bytes == 0 {
        return Err(LpdfError::InvalidInput(
            "Model file is empty (0 bytes)".to_string(),
        ));
    }

    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("model.bin")
        .to_string();

    // Check magic bytes / extension to deduce format
    let mut header = [0u8; 16];
    let mut f = File::open(path).map_err(LpdfError::Io)?;
    let read_bytes = f.read(&mut header).unwrap_or(0);

    let format = if read_bytes >= 4 && &header[0..4] == b"GGUF" {
        "GGUF".to_string()
    } else if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("onnx"))
        .unwrap_or(false)
    {
        "ONNX".to_string()
    } else if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("safetensors"))
        .unwrap_or(false)
    {
        "SafeTensors".to_string()
    } else if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("gguf"))
        .unwrap_or(false)
    {
        "GGUF".to_string()
    } else {
        "Generic Model".to_string()
    };

    let sha256_hex = compute_sha256_hex(path)?;

    Ok(AiModelInfo {
        path: path.to_path_buf(),
        file_name,
        size_bytes,
        sha256_hex,
        format,
        license_spdx: None,
        license_acknowledged: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ts_rs::TS;

    #[test]
    fn test_export_bindings() {
        assert!(AppSettings::export().is_ok());
        assert!(AiModelInfo::export().is_ok());
        assert!(AiModelManifest::export().is_ok());
        assert!(HardwareTier::export().is_ok());
        assert!(OutputPolicy::export().is_ok());
        assert!(ThemePreference::export().is_ok());
    }
}
