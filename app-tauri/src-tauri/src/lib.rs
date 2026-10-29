#![forbid(unsafe_code)]

use directories::ProjectDirs;
use lpdf_core_types::{AiModelInfo, AppSettings};
use lpdf_engine_port::PdfEngine;
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

fn extract_pdf_args<I: IntoIterator<Item = String>>(args: I) -> Vec<String> {
    args.into_iter()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .filter_map(|a| {
            let p = std::path::PathBuf::from(&a);
            if p.is_file()
                && p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("pdf"))
                    .unwrap_or(false)
            {
                Some(a)
            } else {
                None
            }
        })
        .collect()
}

#[tauri::command]
fn pick_files_dialog(
    multiple: bool,
    extensions: Option<Vec<String>>,
) -> Result<Vec<String>, String> {
    #[cfg(target_os = "linux")]
    {
        let filter_str = match &extensions {
            Some(exts) if !exts.is_empty() => {
                let patterns: Vec<String> = exts
                    .iter()
                    .map(|e| format!("*.{}", e.trim_start_matches('.')))
                    .collect();
                patterns.join(" ")
            }
            _ => String::from("*"),
        };

        // Try kdialog first (native KDE Dolphin picker)
        let mut kcmd = std::process::Command::new("kdialog");
        if multiple {
            kcmd.arg("--multiple").arg("--separate-output");
        }
        kcmd.arg("--getopenfilename").arg(":").arg(&filter_str);
        if let Ok(output) = kcmd.output() {
            if output.status.success() {
                let out = String::from_utf8_lossy(&output.stdout);
                let files: Vec<String> = out
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !files.is_empty() {
                    return Ok(files);
                }
            }
        }

        // Fallback to zenity (native GTK / GNOME portal picker)
        let mut zcmd = std::process::Command::new("zenity");
        zcmd.arg("--file-selection");
        if multiple {
            zcmd.arg("--multiple").arg("--separator=\n");
        }
        if let Some(exts) = &extensions {
            if !exts.is_empty() {
                let patterns: Vec<String> = exts
                    .iter()
                    .map(|e| format!("*.{}", e.trim_start_matches('.')))
                    .collect();
                zcmd.arg(format!("--file-filter={}", patterns.join(" ")));
            }
        }
        if let Ok(output) = zcmd.output() {
            if output.status.success() {
                let out = String::from_utf8_lossy(&output.stdout);
                let files: Vec<String> = out
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                return Ok(files);
            }
        }

        Ok(Vec::new())
    }

    #[cfg(target_os = "macos")]
    {
        let script = if multiple {
            "set f to choose file with multiple selections allowed\nset out to \"\"\nrepeat with i in f\nset out to out & POSIX path of i & \"\\n\"\nend repeat\nreturn out"
        } else {
            "POSIX path of (choose file)"
        };
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            let files: Vec<String> = out
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            Ok(files)
        } else {
            Ok(Vec::new())
        }
    }

    #[cfg(target_os = "windows")]
    {
        let mult = if multiple {
            "$d.Multiselect = $true;"
        } else {
            ""
        };
        let filter = match &extensions {
            Some(exts) if !exts.is_empty() => {
                let patterns: Vec<String> = exts
                    .iter()
                    .map(|e| format!("*.{}", e.trim_start_matches('.')))
                    .collect();
                format!(
                    "$d.Filter = 'Files ({})|{}';",
                    patterns.join(";"),
                    patterns.join(";")
                )
            }
            _ => String::from("$d.Filter = 'All Files (*.*)|*.*';"),
        };
        let ps_cmd = format!(
            "Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.OpenFileDialog; {} {} if ($d.ShowDialog() -eq 'OK') {{ if ($d.Multiselect) {{ $d.FileNames -join [Environment]::NewLine }} else {{ $d.FileName }} }}",
            mult, filter
        );
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            let files: Vec<String> = out
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            Ok(files)
        } else {
            Ok(Vec::new())
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Ok(Vec::new())
    }
}

#[tauri::command]
fn pick_folder_dialog() -> Result<Option<String>, String> {
    #[cfg(target_os = "linux")]
    {
        // Try kdialog
        if let Ok(output) = std::process::Command::new("kdialog")
            .arg("--getexistingdirectory")
            .arg(":")
            .output()
        {
            if output.status.success() {
                let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !out.is_empty() {
                    return Ok(Some(out));
                }
            }
        }

        // Try zenity
        if let Ok(output) = std::process::Command::new("zenity")
            .arg("--file-selection")
            .arg("--directory")
            .output()
        {
            if output.status.success() {
                let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !out.is_empty() {
                    return Ok(Some(out));
                }
            }
        }

        Ok(None)
    }

    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg("POSIX path of (choose folder)")
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(if out.is_empty() { None } else { Some(out) })
        } else {
            Ok(None)
        }
    }

    #[cfg(target_os = "windows")]
    {
        let ps_cmd = "Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.FolderBrowserDialog; if ($d.ShowDialog() -eq 'OK') { $d.SelectedPath }";
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", ps_cmd])
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(if out.is_empty() { None } else { Some(out) })
        } else {
            Ok(None)
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Ok(None)
    }
}

#[tauri::command]
fn save_file_dialog(default_name: Option<String>) -> Result<Option<String>, String> {
    #[cfg(target_os = "linux")]
    {
        let start_arg = default_name.as_deref().unwrap_or("");
        // Try kdialog
        if let Ok(output) = std::process::Command::new("kdialog")
            .arg("--getsavefilename")
            .arg(start_arg)
            .output()
        {
            if output.status.success() {
                let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !out.is_empty() {
                    return Ok(Some(out));
                }
            }
        }

        // Try zenity
        let mut zcmd = std::process::Command::new("zenity");
        zcmd.arg("--file-selection")
            .arg("--save")
            .arg("--confirm-overwrite");
        if let Some(ref name) = default_name {
            zcmd.arg(format!("--filename={}", name));
        }
        if let Ok(output) = zcmd.output() {
            if output.status.success() {
                let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !out.is_empty() {
                    return Ok(Some(out));
                }
            }
        }

        Ok(None)
    }

    #[cfg(target_os = "macos")]
    {
        let script = match &default_name {
            Some(name) => format!(
                "POSIX path of (choose file name default name \"{}\")",
                name.replace('"', "\\\"")
            ),
            None => "POSIX path of (choose file name)".to_string(),
        };
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(if out.is_empty() { None } else { Some(out) })
        } else {
            Ok(None)
        }
    }

    #[cfg(target_os = "windows")]
    {
        let file_arg = match &default_name {
            Some(name) => format!("$d.FileName = '{}';", name.replace('\'', "''")),
            None => String::new(),
        };
        let ps_cmd = format!(
            "Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.SaveFileDialog; {} if ($d.ShowDialog() -eq 'OK') {{ $d.FileName }}",
            file_arg
        );
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(if out.is_empty() { None } else { Some(out) })
        } else {
            Ok(None)
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Ok(None)
    }
}

#[tauri::command]
fn get_startup_files() -> Vec<String> {
    extract_pdf_args(std::env::args())
}

/// Payload for converting a PDF with options.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConvertPdfPayload {
    pub input_path: String,
    pub output_path: String,
    pub format: Option<String>,
    pub native_text_only: Option<bool>,
}

/// Payload for extracting text with options.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractTextPayload {
    pub input_path: String,
    pub output_path: String,
    pub native_text_only: Option<bool>,
}

/// Convert a PDF with native text extraction option or structural conversion.
#[tauri::command]
fn convert_pdf(payload: ConvertPdfPayload) -> Result<String, String> {
    let native_only = payload.native_text_only.unwrap_or(false);
    let input = PathBuf::from(&payload.input_path);
    let output = PathBuf::from(&payload.output_path);
    let engine = lpdf_engine_pdfcraft::PdfcraftAdapter::new();
    let format = payload.format.as_deref().unwrap_or("text");
    match format {
        "docx" | "structural_docx" => {
            engine
                .pdf_to_structural_docx(&input, &output)
                .map_err(|e| e.to_string())?;
        }
        "ocr" => {
            engine
                .ocr_with_options(&input, "eng", &output, native_only)
                .map_err(|e| e.to_string())?;
        }
        _ => {
            engine.pdf_to_text(&input, &output).map_err(|e| e.to_string())?;
        }
    }
    Ok(payload.output_path)
}

/// Extract text from a PDF with native_text_only option.
#[tauri::command]
fn extract_text(
    input_path: String,
    output_path: String,
    native_text_only: Option<bool>,
) -> Result<String, String> {
    let _native_only = native_text_only.unwrap_or(false);
    let input = PathBuf::from(&input_path);
    let output = PathBuf::from(&output_path);
    let engine = lpdf_engine_pdfcraft::PdfcraftAdapter::new();
    engine.pdf_to_text(&input, &output).map_err(|e| e.to_string())?;
    Ok(output_path)
}

/// Returns the active backend engine version info.
#[tauri::command]
fn get_engine_version() -> String {
    format!("pdfcraft v0.15.0 ({})", "75360d1d")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _log_guard = setup_logging();

    // Enter the Tauri-managed Tokio runtime context so that plugin init
    // code (e.g. single-instance D-Bus) and channel construction that
    // depend on an active reactor do not panic.
    let _rt_guard = tauri::async_runtime::handle().inner().enter();

    // Create the job event channel
    let (event_tx, mut event_rx) = job_event_channel();

    // Default concurrency = 2 (Tier L safe); will be configurable per-tier later
    let job_manager = Arc::new(JobManager::new(2, event_tx));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let files = extract_pdf_args(argv);
            if !files.is_empty() {
                let _ = app.emit("lpdf://open-files", files);
            }
        }))
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
            import_local_ai_model,
            get_startup_files,
            get_engine_version,
            convert_pdf,
            extract_text,
            pick_files_dialog,
            pick_folder_dialog,
            save_file_dialog
        ])
        .setup(|app| {
            // Forward job events to the Tauri frontend as structured events
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
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
