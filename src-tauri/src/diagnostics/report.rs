use std::fs;
use std::path::PathBuf;

use tauri::Manager;

use super::log::{log_line, log_path};
use crate::frontend::APP_SHELL_ASSET;

fn redact_line(line: &str) -> String {
    let mut out = line.to_string();
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            out = out.replace(&home, "~");
        }
    }
    for pattern in ["Bearer ", "sk-", "api_key", "apikey", "password="] {
        if out.to_lowercase().contains(&pattern.to_lowercase()) {
            out = "[redacted line containing sensitive marker]".into();
            break;
        }
    }
    out
}

pub fn build_summary(
    app: &tauri::AppHandle,
    state: Option<&super::DiagnosticsState>,
) -> Result<String, String> {
    let mut lines = Vec::new();
    lines.push(format!("mochi {}", env!("CARGO_PKG_VERSION")));
    lines.push(format!("platform: {}", platform_name()));
    lines.push(format!("app_shell_asset: {APP_SHELL_ASSET}"));

    if let Ok(config_dir) = app.path().app_config_dir() {
        lines.push(format!(
            "app_config_dir: {}",
            redact_line(&config_dir.display().to_string())
        ));
    }
    if let Ok(log_dir) = app.path().app_log_dir() {
        lines.push(format!(
            "app_log_dir: {}",
            redact_line(&log_dir.display().to_string())
        ));
    }
    if let Some(path) = log_path() {
        lines.push(format!(
            "diagnostics_log: {}",
            redact_line(&path.display().to_string())
        ));
        if let Ok(tail) = read_log_tail(&path, 80) {
            lines.push(String::new());
            lines.push("--- diagnostics.log (tail) ---".into());
            lines.extend(tail.into_iter().map(|line| redact_line(&line)));
        }
    }

    lines.push(String::new());
    lines.push("--- webview windows ---".into());
    for (label, window) in app.webview_windows() {
        let url = window
            .url()
            .map(|parsed| redact_line(parsed.as_ref()))
            .unwrap_or_else(|error| format!("url-error:{error}"));
        let visible = window
            .is_visible()
            .map(|value| value.to_string())
            .unwrap_or_else(|error| format!("visible-error:{error}"));
        lines.push(format!("{label}: url={url} visible={visible}"));
    }

    if let Some(state) = state {
        let inner = state.inner()?;
        if !inner.window_boots.is_empty() {
            lines.push(String::new());
            lines.push("--- frontend boot records ---".into());
            for (label, boot) in &inner.window_boots {
                lines.push(format!(
                    "{label}: path={} target={} tauri={} href={}",
                    boot.pathname,
                    boot.target_route,
                    boot.has_tauri_internals,
                    redact_line(&boot.location_href)
                ));
            }
        }
        if !inner.events.is_empty() {
            lines.push(String::new());
            lines.push("--- recent diagnostic events ---".into());
            for event in inner.events.iter().rev().take(40) {
                lines.push(format!("{}: {}", event.kind, redact_line(&event.detail)));
            }
        }
    }

    Ok(lines.join("\n"))
}

pub fn build_bundle_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let summary = build_summary(app, app.try_state::<super::DiagnosticsState>().as_deref())?;
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&log_dir).map_err(|error| error.to_string())?;
    let timestamp = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".into())
        .replace(':', "-");
    let bundle_path = log_dir.join(format!("mochi-diagnostics-{timestamp}.txt"));
    fs::write(&bundle_path, summary).map_err(|error| error.to_string())?;
    log_line(
        "diagnostics",
        &format!("wrote bundle {}", bundle_path.display()),
    );
    Ok(bundle_path)
}

pub fn run_cli_diagnostics(bundle: bool) -> Result<(), String> {
    super::log::init_cli_log_path()?;
    let mut lines = Vec::new();
    lines.push(format!(
        "mochi {} (CLI diagnostics)",
        env!("CARGO_PKG_VERSION")
    ));
    lines.push(format!("platform: {}", platform_name()));
    lines.push(format!("app_shell_asset: {APP_SHELL_ASSET}"));

    if let Ok(home) = std::env::var("HOME") {
        let config_hint = format!("{home}/.config/mochi");
        lines.push(format!(
            "expected_config_hint: {}",
            redact_line(&config_hint)
        ));
    }

    if let Some(path) = log_path() {
        lines.push(format!(
            "diagnostics_log: {}",
            redact_line(&path.display().to_string())
        ));
        if let Ok(tail) = read_log_tail(&path, 120) {
            lines.push(String::new());
            lines.push("--- diagnostics.log (tail) ---".into());
            lines.extend(tail.into_iter().map(|line| redact_line(&line)));
        }
    } else {
        lines.push("diagnostics_log: (not initialized — run the desktop app once)".into());
    }

    let output = lines.join("\n");
    println!("{output}");

    if bundle {
        let log_dir = log_path()
            .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        let timestamp = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "unknown".into())
            .replace(':', "-");
        let bundle_path = log_dir.join(format!("mochi-diagnostics-cli-{timestamp}.txt"));
        fs::write(&bundle_path, &output).map_err(|error| error.to_string())?;
        println!("\nWrote bundle: {}", bundle_path.display());
    }

    Ok(())
}

fn read_log_tail(path: &PathBuf, max_lines: usize) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let lines: Vec<String> = content.lines().map(str::to_string).collect();
    let start = lines.len().saturating_sub(max_lines);
    Ok(lines[start..].to_vec())
}

fn platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(all(unix, not(target_os = "macos"))) {
        "linux"
    } else {
        "unknown"
    }
}
