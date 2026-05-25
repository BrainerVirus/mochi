use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager};

static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn init_cli_log_path() -> Result<(), String> {
    let log_dir = default_log_dir()?;
    std::fs::create_dir_all(&log_dir).map_err(|error| error.to_string())?;
    set_log_file(log_dir.join("diagnostics.log"))
}

pub fn init(app: &AppHandle) -> Result<(), String> {
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&log_dir).map_err(|error| error.to_string())?;
    set_log_file(log_dir.join("diagnostics.log"))
}

fn set_log_file(path: PathBuf) -> Result<(), String> {
    let mut guard = LOG_PATH
        .lock()
        .map_err(|error| format!("diagnostics log lock poisoned: {error}"))?;
    *guard = Some(path);
    Ok(())
}

fn default_log_dir() -> Result<PathBuf, String> {
    const IDENTIFIER: &str = "app.mochi.Mochi";

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").map_err(|error| error.to_string())?;
        return Ok(PathBuf::from(home)
            .join("Library")
            .join("Logs")
            .join(IDENTIFIER));
    }

    #[cfg(target_os = "windows")]
    {
        let local = std::env::var("LOCALAPPDATA").map_err(|error| error.to_string())?;
        return Ok(PathBuf::from(local).join(IDENTIFIER).join("logs"));
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::var("HOME")
                    .map(|home| PathBuf::from(home).join(".local/share"))
                    .map_err(|error| error.to_string())
            })?;
        return Ok(base.join(IDENTIFIER).join("logs"));
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
    {
        Err("unsupported platform for diagnostics log path".into())
    }
}

pub fn log_path() -> Option<PathBuf> {
    LOG_PATH.lock().ok().and_then(|guard| guard.clone())
}

pub fn log_line(scope: &str, message: &str) {
    let timestamp = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown-time".into());
    let line = format!("[{timestamp}] [{scope}] {message}\n");
    let _ = std::io::stderr().write_all(line.as_bytes());
    if let Some(path) = log_path() {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = file.write_all(line.as_bytes());
        }
    }
}
