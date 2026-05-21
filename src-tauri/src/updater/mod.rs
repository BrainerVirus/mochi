use serde::Serialize;
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub version: Option<String>,
    pub channel: String,
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn check_for_update(
    app: tauri::AppHandle,
    channel: String,
) -> Result<UpdateInfo, String> {
    if std::env::var("FLATPAK_ID").is_ok() {
        return Ok(UpdateInfo {
            available: false,
            version: None,
            channel,
            notes: Some("Flatpak builds use Flatpak-managed updates.".to_string()),
        });
    }

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
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    if std::env::var("FLATPAK_ID").is_ok() {
        return install_flatpak_update().await;
    }

    if let Some(update) = app
        .updater()
        .map_err(|error| error.to_string())?
        .check()
        .await
        .map_err(|error| error.to_string())?
    {
        update
            .download_and_install(|_chunk_length, _content_length| {}, || {})
            .await
            .map_err(|error| error.to_string())?;
        app.restart();
    }

    Ok(())
}

async fn install_flatpak_update() -> Result<(), String> {
    let status = tokio::process::Command::new("flatpak-spawn")
        .args(["--host", "flatpak", "update", "-y", "app.mochi.Mochi"])
        .status()
        .await
        .map_err(|error| format!("failed to start Flatpak update: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Flatpak update exited with status {status}"))
    }
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
}
