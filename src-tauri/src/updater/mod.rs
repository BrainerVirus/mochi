use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

const UPDATE_ENDPOINT_BASE: &str = "https://brainervirus.github.io/mochi/updates";

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
    if channel != "stable" {
        return Err(format!("unsupported update channel: {channel}"));
    }

    reqwest::Url::parse(&format!(
        "{UPDATE_ENDPOINT_BASE}/{{{{target}}}}/{{{{arch}}}}/{{{{current_version}}}}/{channel}.json"
    ))
    .map_err(|error| error.to_string())
}

/// Resolved stable-feed URL for a concrete triple: the same template
/// `check_for_update` hands to the plugin, with placeholders filled in.
pub(crate) fn stable_feed_url(target: &str, arch: &str, current_version: &str) -> String {
    format!("{UPDATE_ENDPOINT_BASE}/{target}/{arch}/{current_version}/stable.json")
}

/// `target-arch` key the stable feed's `platforms` map uses for this host.
fn stable_platform_key() -> Result<String, String> {
    let target = match std::env::consts::OS {
        "macos" => "darwin",
        "windows" => "windows",
        "linux" => "linux",
        other => return Err(format!("unsupported platform for stable updates: {other}")),
    };
    Ok(format!("{target}-{}", std::env::consts::ARCH))
}

fn is_newer_version(feed_version: &str, current_version: &str) -> bool {
    feed_version.trim().trim_start_matches('v') != current_version.trim().trim_start_matches('v')
}

/// Minimal view of the Tauri v2 updater manifest (`stable.json`).
#[derive(Debug, Deserialize)]
struct StableFeed {
    version: String,
    notes: Option<String>,
    platforms: std::collections::HashMap<String, StableFeedPlatform>,
}

#[derive(Debug, Deserialize)]
struct StableFeedPlatform {
    url: String,
}

async fn check_stable_update_async() -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION");
    let key = stable_platform_key()?;
    let target = match key.find('-') {
        Some(index) => &key[..index],
        None => key.as_str(),
    };
    let url = stable_feed_url(target, std::env::consts::ARCH, current);
    let feed: StableFeed = reqwest::get(&url)
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .json()
        .await
        .map_err(|error| error.to_string())?;
    if !is_newer_version(&feed.version, current) {
        return Ok(UpdateInfo {
            available: false,
            version: None,
            channel: "stable".to_string(),
            notes: None,
        });
    }
    // A platform entry without an artifact URL is not installable.
    let artifact_missing = feed
        .platforms
        .get(&key)
        .map(|platform| platform.url.trim().is_empty())
        .unwrap_or(true);
    if artifact_missing {
        return Err(format!("no stable build for {key} in {url}"));
    }
    Ok(UpdateInfo {
        available: true,
        version: Some(feed.version),
        channel: "stable".to_string(),
        notes: feed.notes,
    })
}

/// Non-GUI stable-feed check for the CLI. The Tauri updater plugin needs an
/// `AppHandle`, which does not exist before the Tauri builder runs, so this
/// fetches the same `stable.json` feed `check_for_update` resolves through
/// the plugin and interprets it as the shared `UpdateInfo`.
pub(crate) fn check_stable_update() -> Result<UpdateInfo, String> {
    // Same blocking pattern the CLI refresh path in lib.rs uses.
    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
    runtime.block_on(check_stable_update_async())
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
            "https://brainervirus.github.io/mochi/updates/%7B%7Btarget%7D%7D/%7B%7Barch%7D%7D/%7B%7Bcurrent_version%7D%7D/stable.json"
        );
    }

    #[test]
    fn update_endpoint_rejects_unknown_channel() {
        assert!(update_endpoint_for_channel("nightly").is_err());
        assert!(update_endpoint_for_channel("").is_err());
        let error = update_endpoint_for_channel("beta").expect_err("beta rejected");
        assert!(error.contains("unsupported update channel: beta"));
    }

    #[test]
    fn stable_feed_url_resolves_same_template() {
        assert_eq!(
            stable_feed_url("linux", "x86_64", "0.2.6"),
            "https://brainervirus.github.io/mochi/updates/linux/x86_64/0.2.6/stable.json"
        );
    }

    #[test]
    fn newer_version_ignores_v_prefix_and_whitespace() {
        assert!(!is_newer_version("0.2.6", "0.2.6"));
        assert!(!is_newer_version("v0.2.6", "0.2.6"));
        assert!(is_newer_version("0.2.7", "0.2.6"));
    }
}
