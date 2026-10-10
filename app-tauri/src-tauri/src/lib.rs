use directories::ProjectDirs;
use lpdf_jobs::events::job_event_channel;
use lpdf_jobs::queue::{JobManager, JobRequest};
use std::sync::Arc;
use tauri::Emitter;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Tauri-managed state holding the job manager.
struct AppState {
    job_manager: Arc<JobManager>,
}

/// Submit a job to the background worker queue.
/// Returns the job ID immediately — never blocks the main thread.
#[tauri::command]
fn submit_job(state: tauri::State<'_, AppState>, request: JobRequest) -> String {
    state.job_manager.submit(request)
}

/// Cancel a running or queued job by ID.
#[tauri::command]
fn cancel_job(state: tauri::State<'_, AppState>, job_id: String) -> bool {
    state.job_manager.cancel(&job_id)
}

/// Query the current status of a job by ID.
#[tauri::command]
fn get_job_status(
    state: tauri::State<'_, AppState>,
    job_id: String,
) -> Option<lpdf_jobs::queue::JobStatus> {
    state.job_manager.status(&job_id)
}

/// Read file bytes into frontend memory for PdfViewer.
#[tauri::command]
async fn read_file_bytes(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| e.to_string())
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
            cancel_job,
            get_job_status,
            read_file_bytes
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
