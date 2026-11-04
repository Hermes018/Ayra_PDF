use directories::ProjectDirs;
use lpdf_control_bus::ControlMessage;
use lpdf_fs_safety::atomic_write;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Studio UI preferences persisted atomically to disk (`ADR-012` Phase D1/D2).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioPrefs {
    pub dock_edge: String,
    pub float_x: f64,
    pub float_y: f64,
    pub shortcuts: std::collections::BTreeMap<String, String>,
    pub favorites: Vec<String>,
    pub history_limit: usize,
}

impl Default for StudioPrefs {
    fn default() -> Self {
        Self {
            dock_edge: "left".to_string(),
            float_x: 16.0,
            float_y: 80.0,
            shortcuts: std::collections::BTreeMap::new(),
            favorites: vec![
                "preset.pen.a".to_string(),
                "preset.highlighter".to_string(),
                "preset.eraser.partial".to_string(),
            ],
            history_limit: 500,
        }
    }
}

fn studio_prefs_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "localpdf", "studio") {
        let config_dir = proj_dirs.config_dir();
        let _ = fs::create_dir_all(config_dir);
        config_dir.join("studio_prefs.json")
    } else {
        PathBuf::from("studio_prefs.json")
    }
}

#[tauri::command]
pub fn prefs_load() -> Result<StudioPrefs, String> {
    let path = studio_prefs_path();
    if !path.exists() {
        return Ok(StudioPrefs::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read studio_prefs.json: {}", e))?;
    serde_json::from_str::<StudioPrefs>(&content)
        .map_err(|e| format!("Failed to parse studio_prefs.json: {}", e))
}

#[tauri::command]
pub fn prefs_save(prefs: StudioPrefs) -> Result<(), String> {
    let path = studio_prefs_path();
    let bytes = serde_json::to_vec_pretty(&prefs)
        .map_err(|e| format!("Failed to serialize studio preferences: {}", e))?;
    atomic_write(&path, &bytes)
        .map_err(|e| format!("Failed to atomically write studio_prefs.json: {}", e))
}

#[tauri::command]
pub fn control_validate(msg: ControlMessage) -> Result<bool, String> {
    msg.validate().map(|()| true).map_err(|e| e.to_string())
}
