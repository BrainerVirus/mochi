use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub version: Option<String>,
    pub channel: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateDownloadProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
}

#[tauri::command]
pub async fn check_for_update(
    app: tauri::AppHandle,
    channel: String,
) -> Result<UpdateInfo, String> {
    let update = app
        .updater()
        .map_err(|error| error.to_string())?
        .check()
        .await
        .map_err(|error| error.to_string())?;

    Ok(match update {
        Some(update) => UpdateInfo {
            available: true,
            version: Some(update.version),
            channel,
            notes: update.body,
        },
        None => UpdateInfo {
            available: false,
            version: None,
            channel,
            notes: None,
        },
    })
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    if let Some(update) = app
        .updater()
        .map_err(|error| error.to_string())?
        .check()
        .await
        .map_err(|error| error.to_string())?
    {
        let downloaded = AtomicU64::new(0);
        let app_for_progress = app.clone();

        update
            .download_and_install(
                move |chunk_length, content_length| {
                    let chunk = chunk_length as u64;
                    let next = downloaded.fetch_add(chunk, Ordering::Relaxed) + chunk;
                    let _ = app_for_progress.emit(
                        "update-download-progress",
                        UpdateDownloadProgress {
                            downloaded: next,
                            total: content_length,
                        },
                    );
                },
                || {
                    let _ = app.emit("update-install-started", ());
                },
            )
            .await
            .map_err(|error| error.to_string())?;
        app.restart();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_info_serializes() {
        let info = UpdateInfo {
            available: false,
            version: None,
            channel: "stable".to_string(),
            notes: None,
        };
        let json = serde_json::to_string(&info).expect("serialize");
        assert!(json.contains("stable"));
    }

    #[test]
    fn update_download_progress_serializes() {
        let progress = UpdateDownloadProgress {
            downloaded: 1024,
            total: Some(4096),
        };
        let json = serde_json::to_string(&progress).expect("serialize");
        assert!(json.contains("1024"));
        assert!(json.contains("4096"));
    }
}
