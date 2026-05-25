//! Per-OS browser profile directory roots.
//!
//! `BrowserKind::chromium_support_path` / `gecko_profiles_folder` are relative segments;
//! this module resolves them under the user home (and Windows local app data).

use std::path::{Path, PathBuf};

use crate::browser::catalog::BrowserKind;

/// Chromium user-data directory (contains `Default`, `Profile *`, etc.).
pub fn chromium_user_data_root(home: &Path, browser: BrowserKind) -> Option<PathBuf> {
    let _ = browser.chromium_support_path()?;
    Some(chromium_user_data_root_for(home, browser))
}

fn chromium_user_data_root_for(home: &Path, browser: BrowserKind) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let segment = browser.chromium_support_path().expect("checked");
        home.join("Library/Application Support").join(segment)
    }

    #[cfg(windows)]
    {
        let local = windows_local_app_data(home);
        windows_chromium_root(&local, browser)
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let segment = linux_chromium_config_dir(browser);
        home.join(".config").join(segment)
    }
}

/// Gecko profiles root (`.../Profiles/`).
pub fn gecko_profiles_root(home: &Path, browser: BrowserKind) -> Option<PathBuf> {
    let folder = browser.gecko_profiles_folder()?;
    Some(gecko_profiles_root_for(home, browser, folder))
}

fn gecko_profiles_root_for(home: &Path, browser: BrowserKind, folder: &str) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let _ = browser;
        home.join("Library/Application Support")
            .join(folder)
            .join("Profiles")
    }

    #[cfg(windows)]
    {
        let roaming = windows_roaming_app_data(home);
        match browser {
            BrowserKind::Zen => windows_local_app_data(home).join("zen").join("Profiles"),
            _ => roaming.join("Mozilla").join("Firefox").join("Profiles"),
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let base = match browser {
            BrowserKind::Firefox => home.join(".mozilla").join("firefox"),
            BrowserKind::Zen => home.join(".zen"),
            _ => home.join(".mozilla").join("firefox"),
        };
        base.join("Profiles")
    }
}

#[cfg(windows)]
fn windows_roaming_app_data(home: &Path) -> PathBuf {
    if let Ok(roaming) = std::env::var("APPDATA") {
        if !roaming.trim().is_empty() {
            return PathBuf::from(roaming);
        }
    }
    home.join("AppData").join("Roaming")
}

#[cfg(windows)]
fn windows_local_app_data(home: &Path) -> PathBuf {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        if !local.trim().is_empty() {
            return PathBuf::from(local);
        }
    }
    home.join("AppData").join("Local")
}

#[cfg(windows)]
fn windows_chromium_root(local: &Path, browser: BrowserKind) -> PathBuf {
    match browser {
        BrowserKind::Chrome => local.join("Google").join("Chrome").join("User Data"),
        BrowserKind::ChromeBeta => local.join("Google").join("Chrome Beta").join("User Data"),
        BrowserKind::ChromeCanary => local.join("Google").join("Chrome SxS").join("User Data"),
        BrowserKind::Edge => local.join("Microsoft").join("Edge").join("User Data"),
        BrowserKind::EdgeBeta => local.join("Microsoft").join("Edge Beta").join("User Data"),
        BrowserKind::EdgeCanary => local.join("Microsoft").join("Edge SxS").join("User Data"),
        BrowserKind::Brave => local
            .join("BraveSoftware")
            .join("Brave-Browser")
            .join("User Data"),
        BrowserKind::BraveBeta => local
            .join("BraveSoftware")
            .join("Brave-Browser-Beta")
            .join("User Data"),
        BrowserKind::BraveNightly => local
            .join("BraveSoftware")
            .join("Brave-Browser-Nightly")
            .join("User Data"),
        BrowserKind::Arc => local.join("Arc").join("User Data"),
        BrowserKind::ArcBeta => local.join("Arc Beta").join("User Data"),
        BrowserKind::ArcCanary => local.join("Arc Canary").join("User Data"),
        BrowserKind::Dia => local.join("Dia").join("User Data"),
        BrowserKind::ChatGptAtlas => local
            .join("com.openai.atlas")
            .join("browser-data")
            .join("host"),
        BrowserKind::Chromium => local.join("Chromium").join("User Data"),
        BrowserKind::Helium => local.join("net.imput.helium").join("User Data"),
        BrowserKind::Vivaldi => local.join("Vivaldi").join("User Data"),
        BrowserKind::Yandex => local.join("Yandex").join("YandexBrowser").join("User Data"),
        BrowserKind::Comet => local.join("Comet").join("User Data"),
        _ => {
            let segment = browser.chromium_support_path().unwrap_or("Chromium");
            local.join(segment).join("User Data")
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn linux_chromium_config_dir(browser: BrowserKind) -> &'static str {
    match browser {
        BrowserKind::Chrome => "google-chrome",
        BrowserKind::ChromeBeta => "google-chrome-beta",
        BrowserKind::ChromeCanary => "google-chrome-unstable",
        BrowserKind::Edge => "microsoft-edge",
        BrowserKind::EdgeBeta => "microsoft-edge-beta",
        BrowserKind::EdgeCanary => "microsoft-edge-dev",
        BrowserKind::Brave => "BraveSoftware/Brave-Browser",
        BrowserKind::BraveBeta => "BraveSoftware/Brave-Browser-Beta",
        BrowserKind::BraveNightly => "BraveSoftware/Brave-Browser-Nightly",
        BrowserKind::Arc => "Arc",
        BrowserKind::ArcBeta => "Arc Beta",
        BrowserKind::ArcCanary => "Arc Canary",
        BrowserKind::Dia => "Dia",
        BrowserKind::ChatGptAtlas => "com.openai.atlas/browser-data/host",
        BrowserKind::Chromium => "chromium",
        BrowserKind::Helium => "net.imput.helium",
        BrowserKind::Vivaldi => "vivaldi",
        BrowserKind::Yandex => "yandex-browser",
        BrowserKind::Comet => "Comet",
        _ => "chromium",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_chrome_root_uses_library_application_support() {
        let home = Path::new("/Users/test");
        let root = chromium_user_data_root_for(home, BrowserKind::Chrome);
        assert_eq!(
            root,
            PathBuf::from("/Users/test/Library/Application Support/Google/Chrome")
        );
    }

    #[test]
    #[cfg(all(unix, not(target_os = "macos")))]
    fn linux_chrome_root_uses_dot_config() {
        let home = Path::new("/home/test");
        let root = chromium_user_data_root_for(home, BrowserKind::Chrome);
        assert_eq!(root, PathBuf::from("/home/test/.config/google-chrome"));
    }

    #[test]
    #[cfg(windows)]
    fn windows_chrome_root_uses_local_app_data() {
        let home = Path::new(r"C:\Users\test");
        let root = chromium_user_data_root_for(home, BrowserKind::Chrome);
        assert_eq!(
            root,
            PathBuf::from(r"C:\Users\test\AppData\Local\Google\Chrome\User Data")
        );
    }

    #[test]
    fn macos_gecko_firefox_profiles_root() {
        let home = Path::new("/Users/test");
        let root = gecko_profiles_root_for(home, BrowserKind::Firefox, "Firefox");
        assert_eq!(
            root,
            PathBuf::from("/Users/test/Library/Application Support/Firefox/Profiles")
        );
    }

    #[test]
    #[cfg(all(unix, not(target_os = "macos")))]
    fn linux_zen_profiles_root() {
        let home = Path::new("/home/test");
        let root = gecko_profiles_root(home, BrowserKind::Zen).expect("zen root");
        assert_eq!(root, PathBuf::from("/home/test/.zen/Profiles"));
    }
}
