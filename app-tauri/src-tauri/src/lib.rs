#![forbid(unsafe_code)]

use directories::ProjectDirs;
use lpdf_core_types::{AiModelInfo, AppSettings};
use lpdf_jobs::events::job_event_channel;
use lpdf_jobs::queue::{JobManager, JobPriority, JobRecord, JobRequest, JobStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Emitter;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Tauri-managed state holding the job manager.
struct AppState {
    job_manager: Arc<JobManager>,
}

fn get_settings_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "localpdf") {
        let config_dir = proj_dirs.config_dir();
        let _ = std::fs::create_dir_all(config_dir);
        config_dir.join("settings.toml")
    } else {
        PathBuf::from("settings.toml")
    }
}

/// Submit a job to the background worker queue.
/// Returns the job ID immediately — never blocks the main thread.
#[tauri::command]
fn submit_job(state: tauri::State<'_, AppState>, request: JobRequest) -> String {
    state.job_manager.submit(request)
}

/// Submit a batch of jobs at once.
#[tauri::command]
fn submit_batch(
    state: tauri::State<'_, AppState>,
    requests: Vec<JobRequest>,
    priority: Option<JobPriority>,
) -> Vec<String> {
    state.job_manager.submit_batch(requests, priority)
}

/// Cancel a running or queued job by ID.
#[tauri::command]
fn cancel_job(state: tauri::State<'_, AppState>, job_id: String) -> bool {
    state.job_manager.cancel(&job_id)
}

/// Query the current status of a job by ID.
#[tauri::command]
fn get_job_status(state: tauri::State<'_, AppState>, job_id: String) -> Option<JobStatus> {
    state.job_manager.status(&job_id)
}

/// List all jobs currently tracked in the queue.
#[tauri::command]
fn list_jobs(state: tauri::State<'_, AppState>) -> Vec<JobRecord> {
    state.job_manager.list_jobs()
}

/// Clear finished (completed, failed, cancelled) jobs from memory.
#[tauri::command]
fn clear_finished_jobs(state: tauri::State<'_, AppState>) -> usize {
    state.job_manager.clear_finished_jobs()
}

/// Re-enqueue a failed or cancelled job.
#[tauri::command]
fn retry_job(state: tauri::State<'_, AppState>, job_id: String) -> Result<String, String> {
    state
        .job_manager
        .retry_job(&job_id)
        .map_err(|e| e.to_string())
}

/// List built-in workflow / pipeline presets.
#[tauri::command]
fn list_pipeline_presets() -> Vec<lpdf_jobs::pipeline::PipelinePreset> {
    lpdf_jobs::pipeline::builtin_pipeline_presets()
}

/// Read file bytes into frontend memory for PdfViewer.
#[tauri::command]
async fn read_file_bytes(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| e.to_string())
}

/// Fetch active application settings from local TOML file.
#[tauri::command]
fn get_settings() -> Result<AppSettings, String> {
    let path = get_settings_path();
    lpdf_fs_safety::load_settings(&path).map_err(|e| e.to_string())
}

/// Save application settings to local TOML file atomically via atomic_write.
#[tauri::command]
fn save_settings(settings: AppSettings) -> Result<AppSettings, String> {
    let path = get_settings_path();
    lpdf_fs_safety::save_settings_atomic(&path, &settings).map_err(|e| e.to_string())?;
    Ok(settings)
}

/// Fetch recent log lines from local rotating log files (strictly local).
#[tauri::command]
fn get_recent_logs(max_lines: Option<usize>) -> Result<Vec<String>, String> {
    let limit = max_lines.unwrap_or(200);
    let mut log_lines = Vec::new();

    if let Some(proj_dirs) = ProjectDirs::from("", "", "localpdf") {
        let state_dir = proj_dirs.state_dir().unwrap_or(proj_dirs.data_dir());
        if state_dir.exists() {
            let mut log_files = Vec::new();
            if let Ok(entries) = std::fs::read_dir(state_dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with("localpdf.log") {
                        if let Ok(metadata) = entry.metadata() {
                            let mtime = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
                            log_files.push((entry.path(), mtime));
                        }
                    }
                }
            }

            log_files.sort_by_key(|k| k.1);

            for (path, _) in log_files {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            log_lines.push(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }

    if log_lines.is_empty() {
        log_lines
            .push("[INFO] Local rotating logging active. No archived entries yet.".to_string());
    }

    let start = log_lines.len().saturating_sub(limit);
    Ok(log_lines[start..].to_vec())
}

/// Export a sanitized local diagnostics report atomically to the chosen output path.
#[tauri::command]
fn export_diagnostics(output_path: String) -> Result<String, String> {
    let target = PathBuf::from(&output_path);
    let hw = lpdf_hw::HardwareProfile::detect();
    let settings = get_settings().unwrap_or_default();
    let logs = get_recent_logs(Some(100)).unwrap_or_default();

    let mut report = String::new();
    report.push_str("=== LOCALPDF STUDIO DIAGNOSTICS REPORT (LOCAL ONLY) ===\n");
    report.push_str(&format!("Generated: {:?}\n", std::time::SystemTime::now()));
    report.push_str(&format!(
        "OS / Architecture: {} / {}\n",
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    report.push_str(&format!("Detected Hardware Tier: {:?}\n", hw.tier));
    report.push_str(&format!("Total Memory: {} MB\n", hw.total_memory_mb));
    report.push_str(&format!("Logical CPU Cores: {}\n", hw.logical_cores));
    report.push_str(&format!(
        "Active Tier Concurrency: {} jobs\n",
        hw.concurrent_jobs()
    ));
    report.push_str(&format!("Active Tile Cache: {} MB\n\n", hw.tile_cache_mb()));

    report.push_str("--- Active Settings ---\n");
    report.push_str(&format!("Tier Override: {:?}\n", settings.tier_override));
    report.push_str(&format!("Output Policy: {:?}\n", settings.output_policy));
    report.push_str(&format!("Backups Enabled: {}\n", settings.backups_enabled));
    report.push_str(&format!("Language: {}\n", settings.language));
    report.push_str(&format!("Theme: {:?}\n", settings.theme));
    report.push_str(&format!(
        "AI Local Enabled: {}\n",
        settings.ai_local_enabled
    ));
    if let Some(ref model_path) = settings.local_ai_model_path {
        let file_name = model_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        report.push_str(&format!("Local AI Model: {}\n", file_name));
    } else {
        report.push_str("Local AI Model: None\n");
    }
    report.push_str("\n--- Recent Log Tail (Sanitized) ---\n");
    for line in logs {
        report.push_str(&line);
        report.push('\n');
    }

    lpdf_fs_safety::atomic_write(&target, report.as_bytes()).map_err(|e| e.to_string())?;

    Ok(target.to_string_lossy().to_string())
}

/// Import a local AI model file (.gguf, .onnx, etc.), verify existence, header, and SHA-256, and update AppSettings.
/// Requires explicit license acknowledgement before enabling `ai_local_enabled` (Failsafe F19).
#[tauri::command]
fn import_local_ai_model(
    model_path: String,
    license_spdx: Option<String>,
    acknowledge_license: Option<bool>,
) -> Result<AiModelInfo, String> {
    let path = PathBuf::from(&model_path);
    let mut info = lpdf_core_types::inspect_ai_model(&path).map_err(|e| e.to_string())?;

    let ack = acknowledge_license.unwrap_or(false);
    info.license_spdx = license_spdx.clone();
    info.license_acknowledged = ack;

    let mut current_settings = get_settings().unwrap_or_default();
    current_settings.local_ai_model_path = Some(path);
    current_settings.ai_model_sha256 = Some(info.sha256_hex.clone());
    current_settings.ai_model_license = license_spdx;
    current_settings.ai_license_acknowledged = ack;
    // Failsafe F19: refuse to enable ai_local_enabled unless license is explicitly acknowledged
    current_settings.ai_local_enabled = ack;
    let _ = save_settings(current_settings);

    Ok(info)
}

fn setup_logging() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "localpdf") {
        let state_dir = proj_dirs.state_dir().unwrap_or(proj_dirs.data_dir());
        std::fs::create_dir_all(state_dir).ok()?;
        let file_appender = tracing_appender::rolling::daily(state_dir, "localpdf.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
            .with(fmt::layer().with_writer(std::io::stdout))
            .init();

        Some(guard)
    } else {
        None
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _log_guard = setup_logging();

    // Create the job event channel
    let (event_tx, mut event_rx) = job_event_channel();

    // Default concurrency = 2 (Tier L safe); will be configurable per-tier later
    let job_manager = Arc::new(JobManager::new(2, event_tx));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .manage(AppState {
            job_manager: Arc::clone(&job_manager),
        })
        .invoke_handler(tauri::generate_handler![
            submit_job,
            submit_batch,
            cancel_job,
            get_job_status,
            list_jobs,
            clear_finished_jobs,
            retry_job,
            list_pipeline_presets,
            read_file_bytes,
            get_settings,
            save_settings,
            get_recent_logs,
            export_diagnostics,
            import_local_ai_model
        ])
        .setup(|app| {
            // Forward job events to the Tauri frontend as structured events
            let handle = app.handle().clone();
            tokio::spawn(async move {
                while let Some(event) = event_rx.recv().await {
                    tracing::debug!("Job event: {:?}", event);
                    // Emit to all windows; the frontend filters by job_id
                    let _ = handle.emit("job-event", &event);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
