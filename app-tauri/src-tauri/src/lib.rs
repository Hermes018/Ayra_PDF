use directories::ProjectDirs;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
