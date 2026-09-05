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
    /// Installer artifact URL from the stable feed's platform entry. `None`
    /// on the plugin path (which never exposes it) and when up to date.
    pub download_url: Option<String>,
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
        .map_err(|error| {
            let message = error.to_string();
            crate::diagnostics::log_line("update", &format!("check failed: {message}"));
            message
        })?;

    Ok(match update {
        Some(update) => {
            crate::diagnostics::log_line(
                "update",
                &format!("check ok: {} available (channel {channel})", update.version),
            );
            UpdateInfo {
                available: true,
                version: Some(update.version),
                channel,
                notes: update.body,
                download_url: None,
            }
        }
        None => {
            crate::diagnostics::log_line("update", "check ok: up to date");
            UpdateInfo {
                available: false,
                version: None,
                channel,
                notes: None,
                download_url: None,
            }
        }
    })
}

#[tauri::command]
pub async fn install_update(app: AppHandle, channel: String) -> Result<(), String> {
    if let Some(update) = updater_for_channel(&app, &channel)?
        .check()
        .await
        .map_err(|error| {
            let message = error.to_string();
            crate::diagnostics::log_line("update", &format!("install check failed: {message}"));
            message
        })?
    {
        let version = update.version.clone();
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
            .map_err(|error| {
                let message = error.to_string();
                crate::diagnostics::log_line(
                    "update",
                    &format!("install failed for {version}: {message}"),
                );
                message
            })?;
        crate::diagnostics::log_line("update", &format!("install ok: {version}, restarting"));
        crate::notifications::send_notification(
            &app,
            "Mochi update ready",
            &format!("Version {version} installed"),
        );
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
/// `base` is injectable so tests can point at a local fake server; production
/// passes `UPDATE_ENDPOINT_BASE`.
pub(crate) fn stable_feed_url_with_base(
    base: &str,
    target: &str,
    arch: &str,
    current_version: &str,
) -> String {
    format!("{base}/{target}/{arch}/{current_version}/stable.json")
}

/// `(target, arch)` pair the stable feed's `platforms` map keys as
/// `target-arch` for this host.
fn stable_platform() -> Result<(&'static str, &'static str), String> {
    let target = match std::env::consts::OS {
        "macos" => "darwin",
        "windows" => "windows",
        "linux" => "linux",
        other => return Err(format!("unsupported platform for stable updates: {other}")),
    };
    Ok((target, std::env::consts::ARCH))
}

/// Numeric dotted-version ordering (`0.10.0` beats `0.9.9`; an older feed
/// version is never "newer"). Non-numeric suffixes (pre-release tags) read
/// as 0 so they never outrank a release.
fn numeric_version_parts(version: &str) -> Vec<u64> {
    version
        .trim()
        .trim_start_matches('v')
        .split('.')
        .map(|part| {
            part.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<u64>()
                .unwrap_or(0)
        })
        .collect()
}

fn is_newer_version(feed_version: &str, current_version: &str) -> bool {
    let (mut feed, mut current) = (
        numeric_version_parts(feed_version),
        numeric_version_parts(current_version),
    );
    let width = feed.len().max(current.len());
    feed.resize(width, 0);
    current.resize(width, 0);
    feed > current
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

async fn check_stable_update_async_with_base(base: &str) -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION");
    let (target, arch) = stable_platform()?;
    let key = format!("{target}-{arch}");
    let url = stable_feed_url_with_base(base, target, arch, current);
    let feed: StableFeed = reqwest::get(&url)
        .await
        .map_err(|error| format!("failed to fetch stable feed {url}: {error}"))?
        .error_for_status()
        .map_err(|error| format!("failed to fetch stable feed {url}: {error}"))?
        .json()
        .await
        .map_err(|error| format!("failed to parse stable feed {url}: {error}"))?;
    if !is_newer_version(&feed.version, current) {
        return Ok(UpdateInfo {
            available: false,
            version: None,
            channel: "stable".to_string(),
            notes: None,
            download_url: None,
        });
    }
    // A platform entry without an artifact URL is not installable.
    let artifact_url = feed
        .platforms
        .get(&key)
        .map(|platform| platform.url.trim().to_string())
        .filter(|url| !url.is_empty());
    match artifact_url {
        Some(artifact_url) => Ok(UpdateInfo {
            available: true,
            version: Some(feed.version),
            channel: "stable".to_string(),
            notes: feed.notes,
            download_url: Some(artifact_url),
        }),
        None => Err(format!("no stable build for {key} in {url}")),
    }
}

async fn check_stable_update_async() -> Result<UpdateInfo, String> {
    check_stable_update_async_with_base(UPDATE_ENDPOINT_BASE).await
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
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    /// One-request test HTTP server: accepts a single connection, consumes
    /// the request headers, writes a canned response, then shuts down.
    fn serve_one(status_line: &str, body: String) -> (String, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let addr = listener.local_addr().expect("test server addr").to_string();
        let status_line = status_line.to_string();
        let handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept test connection");
            let mut reader = BufReader::new(stream.try_clone().expect("clone test stream"));
            let mut line = String::new();
            loop {
                line.clear();
                let read = reader.read_line(&mut line).expect("read test request");
                if read == 0 || line == "\r\n" {
                    break;
                }
            }
            let response = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let mut stream = stream;
            stream
                .write_all(response.as_bytes())
                .expect("write test response");
        });
        (addr, handle)
    }

    fn host_platform_key() -> String {
        let (target, arch) = stable_platform().expect("host platform");
        format!("{target}-{arch}")
    }

    fn feed_body(version: &str, key: &str, with_artifact: bool) -> String {
        let platforms = if with_artifact {
            format!("{{\"{key}\":{{\"url\":\"https://example.com/mochi-{version}.dmg\"}}}}")
        } else {
            "{}".to_string()
        };
        format!("{{\"version\":\"{version}\",\"notes\":\"- fixes\",\"platforms\":{platforms}}}")
    }

    #[test]
    fn update_info_serializes() {
        let info = UpdateInfo {
            available: false,
            version: None,
            channel: "stable".to_string(),
            notes: None,
            download_url: None,
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
            stable_feed_url_with_base(UPDATE_ENDPOINT_BASE, "linux", "x86_64", "0.2.6"),
            "https://brainervirus.github.io/mochi/updates/linux/x86_64/0.2.6/stable.json"
        );
    }

    #[test]
    fn newer_version_ignores_v_prefix_and_whitespace() {
        assert!(!is_newer_version("0.2.6", "0.2.6"));
        assert!(!is_newer_version("v0.2.6", "0.2.6"));
        assert!(is_newer_version("0.2.7", "0.2.6"));
    }

    #[test]
    fn older_feed_version_is_not_newer() {
        assert!(!is_newer_version("0.2.5", "0.2.6"));
        assert!(!is_newer_version("v0.2.5", "0.2.6"));
        assert!(is_newer_version("0.10.0", "0.9.9"));
    }

    #[test]
    fn stable_platform_returns_target_and_arch() {
        let (target, arch) = stable_platform().expect("host platform");
        assert!(!target.is_empty());
        assert_eq!(arch, std::env::consts::ARCH);
    }

    #[tokio::test]
    async fn stable_check_success_returns_version_notes_and_url() {
        let key = host_platform_key();
        let (addr, server) = serve_one("200 OK", feed_body("99.0.0", &key, true));
        let info = check_stable_update_async_with_base(&format!("http://{addr}"))
            .await
            .expect("check must succeed");
        assert!(info.available);
        assert_eq!(info.version.as_deref(), Some("99.0.0"));
        assert_eq!(info.notes.as_deref(), Some("- fixes"));
        assert!(
            info.download_url
                .as_deref()
                .is_some_and(|url| url.contains("99.0.0")),
            "must carry the installer URL, got {:?}",
            info.download_url
        );
        server.join().expect("server thread");
    }

    #[tokio::test]
    async fn stable_check_network_failure_names_feed_url() {
        let addr = TcpListener::bind("127.0.0.1:0")
            .expect("bind")
            .local_addr()
            .expect("addr");
        // Listener is dropped: nothing answers, so the check fails refused.
        let base = format!("http://{addr}");
        let error = check_stable_update_async_with_base(&base)
            .await
            .expect_err("refused connection must fail");
        assert!(
            error.contains("failed to fetch stable feed"),
            "actionable prefix, got: {error}"
        );
        assert!(error.contains(&base), "must name the feed, got: {error}");
    }

    #[tokio::test]
    async fn stable_check_malformed_feed_is_parse_error() {
        let (addr, server) = serve_one("200 OK", "not-json{".to_string());
        let base = format!("http://{addr}");
        let error = check_stable_update_async_with_base(&base)
            .await
            .expect_err("malformed feed must fail");
        assert!(
            error.contains("failed to parse stable feed"),
            "parse error, got: {error}"
        );
        assert!(error.contains(&base), "must name the feed, got: {error}");
        server.join().expect("server thread");
    }

    #[tokio::test]
    async fn stable_check_missing_artifact_errors() {
        let key = host_platform_key();
        let (addr, server) = serve_one("200 OK", feed_body("99.0.0", &key, false));
        let error = check_stable_update_async_with_base(&format!("http://{addr}"))
            .await
            .expect_err("missing artifact must fail");
        assert!(error.contains("no stable build"), "got: {error}");
        assert!(error.contains(&key), "must name the platform, got: {error}");
        server.join().expect("server thread");
    }

    #[tokio::test]
    async fn stable_check_older_feed_version_is_up_to_date() {
        let key = host_platform_key();
        let (addr, server) = serve_one("200 OK", feed_body("0.0.1", &key, true));
        let info = check_stable_update_async_with_base(&format!("http://{addr}"))
            .await
            .expect("check must succeed");
        assert!(!info.available);
        assert!(info.version.is_none());
        server.join().expect("server thread");
    }
}
