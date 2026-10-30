#![cfg_attr(not(target_os = "windows"), forbid(unsafe_code))]

use lpdf_core_types::{AppSettings, LpdfError, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
mod win_fs {
    #![allow(unsafe_code)]
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use std::time::Duration;
    use windows_sys::Win32::Foundation::{
        GetLastError, ERROR_ACCESS_DENIED, ERROR_LOCK_VIOLATION, ERROR_SHARING_VIOLATION,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    fn to_wide_null(p: &Path) -> Vec<u16> {
        p.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    pub fn atomic_replace_with_backoff(temp_path: &Path, target_path: &Path) -> std::io::Result<()> {
        let src_w = to_wide_null(temp_path);
        let dst_w = to_wide_null(target_path);
        let retry_delays_ms = [25_u64, 50, 100, 200, 400, 800];

        for (attempt, delay_ms) in retry_delays_ms.iter().enumerate() {
            let ok = unsafe {
                MoveFileExW(
                    src_w.as_ptr(),
                    dst_w.as_ptr(),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            };
            if ok != 0 {
                return Ok(());
            }

            let err_code = unsafe { GetLastError() };
            let is_transient_lock = err_code == ERROR_SHARING_VIOLATION
                || err_code == ERROR_LOCK_VIOLATION
                || err_code == ERROR_ACCESS_DENIED;

            if !is_transient_lock || attempt == retry_delays_ms.len() - 1 {
                return Err(std::io::Error::from_raw_os_error(err_code as i32));
            }

            std::thread::sleep(Duration::from_millis(*delay_ms));
        }

        Err(std::io::Error::other("Exhausted atomic replace retries"))
    }
}

#[cfg(target_os = "windows")]
pub mod win_sandbox {
    #![allow(unsafe_code)]
    use std::os::windows::io::AsRawHandle;
    use std::os::windows::process::CommandExt;
    use std::process::{Child, Command};
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION, JOB_OBJECT_LIMIT_JOB_MEMORY,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    pub const CREATE_NO_WINDOW_FLAG: u32 = 0x0800_0000;
    pub const SIDECAR_MEMORY_LIMIT_BYTES: usize = 1024 * 1024 * 1024; // 1 GiB per ADR-001

    /// Configures `std::process::Command` on Windows to suppress console window flashing.
    pub fn apply_no_window_flags(cmd: &mut Command) {
        cmd.creation_flags(CREATE_NO_WINDOW_FLAG);
    }

    /// RAII Win32 Job Object that enforces a 1 GiB memory ceiling and guarantees
    /// all assigned child/grandchild processes are terminated when dropped or if the host crashes.
    pub struct WinJobSandbox {
        job_handle: HANDLE,
    }

    // Safety: Win32 kernel job handles are thread-safe.
    unsafe impl Send for WinJobSandbox {}
    unsafe impl Sync for WinJobSandbox {}

    impl WinJobSandbox {
        pub fn new_with_memory_cap(memory_limit_bytes: usize) -> std::io::Result<Self> {
            let job_handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
            if job_handle.is_null() {
                return Err(std::io::Error::last_os_error());
            }

            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                | JOB_OBJECT_LIMIT_JOB_MEMORY
                | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION;
            info.JobMemoryLimit = memory_limit_bytes;

            let ok = unsafe {
                SetInformationJobObject(
                    job_handle,
                    JobObjectExtendedLimitInformation,
                    &info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION as *const std::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            };

            if ok == 0 {
                let err = std::io::Error::last_os_error();
                unsafe {
                    let _ = CloseHandle(job_handle);
                }
                return Err(err);
            }

            Ok(Self { job_handle })
        }

        pub fn assign_child(&self, child: &Child) -> std::io::Result<()> {
            let process_handle = child.as_raw_handle() as HANDLE;
            let ok = unsafe { AssignProcessToJobObject(self.job_handle, process_handle) };
            if ok == 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        }
    }

    impl Drop for WinJobSandbox {
        fn drop(&mut self) {
            if !self.job_handle.is_null() {
                unsafe {
                    let _ = CloseHandle(self.job_handle);
                }
            }
        }
    }
}

/// Cross-platform process sandbox abstraction enforcing memory ceilings and execution containment.
pub struct ChildProcessSandbox {
    #[cfg(target_os = "windows")]
    inner: Option<win_sandbox::WinJobSandbox>,
}

impl ChildProcessSandbox {
    pub fn new_sidecar_limit() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self {
                inner: win_sandbox::WinJobSandbox::new_with_memory_cap(
                    win_sandbox::SIDECAR_MEMORY_LIMIT_BYTES,
                )
                .ok(),
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self {}
        }
    }

    pub fn configure_command(cmd: &mut std::process::Command) {
        #[cfg(target_os = "windows")]
        win_sandbox::apply_no_window_flags(cmd);
        #[cfg(not(target_os = "windows"))]
        let _ = cmd;
    }

    pub fn attach(&self, child: &std::process::Child) {
        #[cfg(target_os = "windows")]
        if let Some(ref job) = self.inner {
            let _ = job.assign_child(child);
        }
        #[cfg(not(target_os = "windows"))]
        let _ = child;
    }
}

/// Simplifies paths for external CLI sidecars (such as LibreOffice or Pandoc).
/// On Windows, converts verbatim extended-length prefix (`\\?\C:\...`) into standard
/// Win32 path format (`C:\...`) using `dunce::simplified` so external CLI parsers do not choke.
/// On non-Windows platforms, returns the path as-is.
pub fn simplify_path_for_sidecar(path: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dunce::simplified(path).to_path_buf()
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.to_path_buf()
    }
}

/// Formats a local path into a file URL (`file://...`).
/// On Windows, normalizes backslashes to forward slashes (e.g., `file:///C:/Users/...`).
/// On non-Windows, prepends `file://` to the absolute path string.
pub fn format_file_url_from_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    #[cfg(target_os = "windows")]
    {
        let normalized = path_str.replace('\\', "/");
        if normalized.starts_with('/') {
            format!("file://{}", normalized)
        } else {
            format!("file:///{}", normalized)
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        format!("file://{}", path_str)
    }
}

/// Safely writes bytes to `target_path` atomically using a temporary file
/// in the same directory, verifying byte count, syncing to disk, and renaming.
pub fn atomic_write(target_path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(LpdfError::Io)?;

    let file_stem = target_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = target_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("tmp");

    let temp_name = format!(".{}_{}_{}.tmp", file_stem, ext, uuid_simple());
    let temp_path = parent.join(temp_name);

    {
        let mut file = fs::File::create(&temp_path).map_err(LpdfError::Io)?;
        file.write_all(bytes).map_err(LpdfError::Io)?;
        file.sync_all().map_err(LpdfError::Io)?;
    }

    let meta = fs::metadata(&temp_path).map_err(LpdfError::Io)?;
    if meta.len() != bytes.len() as u64 {
        let _ = fs::remove_file(&temp_path);
        return Err(LpdfError::Io(std::io::Error::other(
            "Written byte count mismatch",
        )));
    }

    #[cfg(target_os = "windows")]
    {
        match win_fs::atomic_replace_with_backoff(&temp_path, target_path) {
            Ok(()) => Ok(()),
            Err(e) => {
                // Recovery failsafe: if target locked, save with unique suffix to avoid data loss
                let recovery_name = format!("{}_saved_{}.{}", file_stem, uuid_simple(), ext);
                let recovery_path = parent.join(recovery_name);
                if fs::rename(&temp_path, &recovery_path).is_ok() {
                    Err(LpdfError::Io(std::io::Error::other(format!(
                        "Target file locked; safely saved recovery copy to '{}': {}",
                        recovery_path.display(),
                        e
                    ))))
                } else {
                    Err(LpdfError::Io(e))
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        fs::rename(&temp_path, target_path).map_err(LpdfError::Io)?;
        Ok(())
    }
}

/// Persists `AppSettings` to a TOML file atomically via `atomic_write`.
pub fn save_settings_atomic(path: &Path, settings: &AppSettings) -> Result<()> {
    let toml_str =
        toml::to_string_pretty(settings).map_err(|e| LpdfError::ConfigError(e.to_string()))?;
    atomic_write(path, toml_str.as_bytes())
}

/// Loads `AppSettings` from a TOML file, returning `AppSettings::default()` if the file does not exist.
pub fn load_settings(path: &Path) -> Result<AppSettings> {
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let content = fs::read_to_string(path).map_err(LpdfError::Io)?;
    let settings: AppSettings =
        toml::from_str(&content).map_err(|e| LpdfError::ConfigError(e.to_string()))?;
    Ok(settings)
}

/// Create a safely named, newly created output folder inside `parent_dir` based on `prefix`.
/// If the folder already exists, appends numbers (_1, _2, etc.) to prevent collisions.
pub fn create_safe_output_dir(parent_dir: &Path, prefix: &str) -> Result<PathBuf> {
    fs::create_dir_all(parent_dir).map_err(LpdfError::Io)?;
    let sanitized_prefix = sanitize_filename(prefix);

    let mut candidate = parent_dir.join(&sanitized_prefix);
    let mut counter = 1;
    while candidate.exists() {
        candidate = parent_dir.join(format!("{}_{}", sanitized_prefix, counter));
        counter += 1;
    }

    fs::create_dir_all(&candidate).map_err(LpdfError::Io)?;
    Ok(candidate)
}

/// Sanitize filename to avoid directory traversal or reserved characters.
pub fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('_').trim();
    if trimmed.is_empty() {
        "export".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Generate a short random alphanumeric string for temp file suffixes without extra external deps.
pub fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}
