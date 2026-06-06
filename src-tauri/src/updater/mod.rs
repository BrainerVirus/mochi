use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

const UPDATE_ENDPOINT_BASE: &str = "https://mochi-app.github.io/mochi/updates";

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
    let update = updater_for_channel(&app, &channel)?
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
pub async fn install_update(app: AppHandle, channel: String) -> Result<(), String> {
    if let Some(update) = updater_for_channel(&app, &channel)?
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

fn updater_for_channel(
    app: &AppHandle,
    channel: &str,
) -> Result<tauri_plugin_updater::Updater, String> {
    let endpoint = update_endpoint_for_channel(channel)?;
    app.updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|error| error.to_string())?
        .build()
        .map_err(|error| error.to_string())
}

fn update_endpoint_for_channel(channel: &str) -> Result<reqwest::Url, String> {
    let channel = match channel {
        "stable" | "unstable" => channel,
        other => return Err(format!("unsupported update channel: {other}")),
    };

    reqwest::Url::parse(&format!(
        "{UPDATE_ENDPOINT_BASE}/{{{{target}}}}/{{{{arch}}}}/{{{{current_version}}}}/{channel}.json"
    ))
    .map_err(|error| error.to_string())
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

    #[test]
    fn update_endpoint_builds_exact_stable_feed_url() {
        let endpoint = update_endpoint_for_channel("stable").expect("stable endpoint");
        assert_eq!(
            endpoint.as_str(),
            "https://mochi-app.github.io/mochi/updates/%7B%7Btarget%7D%7D/%7B%7Barch%7D%7D/%7B%7Bcurrent_version%7D%7D/stable.json"
        );
    }

    #[test]
    fn update_endpoint_builds_exact_unstable_feed_url() {
        let endpoint = update_endpoint_for_channel("unstable").expect("unstable endpoint");
        assert_eq!(
            endpoint.as_str(),
            "https://mochi-app.github.io/mochi/updates/%7B%7Btarget%7D%7D/%7B%7Barch%7D%7D/%7B%7Bcurrent_version%7D%7D/unstable.json"
        );
    }

    #[test]
    fn update_endpoint_rejects_unknown_channel() {
        let error = update_endpoint_for_channel("beta").expect_err("beta rejected");
        assert!(error.contains("unsupported update channel: beta"));
    }
}
