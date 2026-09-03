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

pub fn chromium_user_data_roots(home: &Path, browser: BrowserKind) -> Vec<PathBuf> {
    let Some(primary) = chromium_user_data_root(home, browser) else {
        return Vec::new();
    };

    let mut roots = vec![primary];
    roots.extend(extra_chromium_user_data_roots(home, browser));
    dedupe_roots(roots)
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

/// Gecko profiles root.
pub fn gecko_profiles_root(home: &Path, browser: BrowserKind) -> Option<PathBuf> {
    let folder = browser.gecko_profiles_folder()?;
    Some(gecko_profiles_root_for(home, browser, folder))
}

pub fn gecko_profiles_roots(home: &Path, browser: BrowserKind) -> Vec<PathBuf> {
    let Some(primary) = gecko_profiles_root(home, browser) else {
        return Vec::new();
    };

    let mut roots = vec![primary];
    roots.extend(extra_gecko_profiles_roots(home, browser));

    dedupe_roots(roots)
}

fn dedupe_roots(roots: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut deduped = Vec::new();
    for root in roots {
        if !deduped.contains(&root) {
            deduped.push(root);
        }
    }
    deduped
}

fn extra_chromium_user_data_roots(home: &Path, browser: BrowserKind) -> Vec<PathBuf> {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let segment = linux_chromium_config_dir(browser);
        let mut roots = Vec::new();

        for app_id in linux_flatpak_app_ids(browser) {
            roots.push(linux_flatpak_config_root(home, app_id, segment));
            roots.push(linux_flatpak_dot_config_root(home, app_id, segment));
        }

        for snap_name in linux_snap_names(browser) {
            roots.push(linux_snap_common_config_root(home, snap_name, segment));
            roots.push(linux_snap_current_config_root(home, snap_name, segment));
        }

        roots
    }

    #[cfg(not(all(unix, not(target_os = "macos"))))]
    {
        let _ = home;
        let _ = browser;
        Vec::new()
    }
}

fn extra_gecko_profiles_roots(home: &Path, browser: BrowserKind) -> Vec<PathBuf> {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let roots = match browser {
            BrowserKind::Firefox => vec![
                linux_flatpak_home_relative_root(home, "org.mozilla.firefox", ".mozilla/firefox"),
                linux_flatpak_config_root(home, "org.mozilla.firefox", "mozilla/firefox"),
                linux_flatpak_dot_config_root(home, "org.mozilla.firefox", "mozilla/firefox"),
                linux_snap_home_relative_root(home, "firefox", ".mozilla/firefox"),
            ],
            BrowserKind::Zen => {
                let mut roots: Vec<PathBuf> = linux_zen_flatpak_app_ids()
                    .iter()
                    .flat_map(|app_id| {
                        [
                            linux_flatpak_home_relative_root(home, app_id, ".zen"),
                            linux_flatpak_home_relative_root(home, app_id, "zen"),
                            linux_flatpak_config_root(home, app_id, "zen"),
                            linux_flatpak_dot_config_root(home, app_id, "zen"),
                        ]
                    })
                    .collect();
                for snap_name in linux_zen_snap_names() {
                    roots.push(linux_snap_home_relative_root(home, snap_name, ".zen"));
                    roots.push(linux_snap_home_relative_root(home, snap_name, "zen"));
                    roots.push(linux_snap_common_config_root(home, snap_name, "zen"));
                }
                roots
            }
            _ => Vec::new(),
        };

        roots
    }

    #[cfg(not(all(unix, not(target_os = "macos"))))]
    {
        let _ = home;
        let _ = browser;
        Vec::new()
    }
}

#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
fn linux_flatpak_config_root(home: &Path, app_id: &str, config_segment: &str) -> PathBuf {
    home.join(".var")
        .join("app")
        .join(app_id)
        .join("config")
        .join(config_segment)
}

#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
fn linux_flatpak_dot_config_root(home: &Path, app_id: &str, config_segment: &str) -> PathBuf {
    linux_flatpak_home_relative_root(home, app_id, &format!(".config/{config_segment}"))
}

fn linux_flatpak_home_relative_root(home: &Path, app_id: &str, relative: &str) -> PathBuf {
    home.join(".var").join("app").join(app_id).join(relative)
}

#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
fn linux_snap_common_config_root(home: &Path, snap_name: &str, config_segment: &str) -> PathBuf {
    linux_snap_home_relative_root(home, snap_name, &format!(".config/{config_segment}"))
}

#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
fn linux_snap_current_config_root(home: &Path, snap_name: &str, config_segment: &str) -> PathBuf {
    home.join("snap")
        .join(snap_name)
        .join("current")
        .join(".config")
        .join(config_segment)
}

#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
fn linux_snap_home_relative_root(home: &Path, snap_name: &str, relative: &str) -> PathBuf {
    home.join("snap")
        .join(snap_name)
        .join("common")
        .join(relative)
}

#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
fn linux_flatpak_app_ids(browser: BrowserKind) -> &'static [&'static str] {
    match browser {
        BrowserKind::Chrome => &["com.google.Chrome"],
        BrowserKind::ChromeBeta => &["com.google.Chrome.Beta"],
        BrowserKind::ChromeCanary => &["com.google.ChromeDev"],
        BrowserKind::Edge => &["com.microsoft.Edge"],
        BrowserKind::EdgeBeta => &["com.microsoft.Edge.Beta"],
        BrowserKind::EdgeCanary => &["com.microsoft.EdgeDev"],
        BrowserKind::Brave => &["com.brave.Browser"],
        BrowserKind::BraveBeta => &["com.brave.Browser.Beta"],
        BrowserKind::BraveNightly => &["com.brave.Browser.Nightly"],
        BrowserKind::Chromium => &["org.chromium.Chromium"],
        BrowserKind::Helium => &["net.imput.helium"],
        BrowserKind::Vivaldi => &["com.vivaldi.Vivaldi"],
        BrowserKind::Yandex => &["ru.yandex.Browser"],
        _ => &[],
    }
}

#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
fn linux_snap_names(browser: BrowserKind) -> &'static [&'static str] {
    match browser {
        BrowserKind::Chrome => &["google-chrome"],
        BrowserKind::ChromeBeta => &["google-chrome-beta"],
        BrowserKind::ChromeCanary => &["google-chrome-unstable"],
        BrowserKind::Edge => &["microsoft-edge"],
        BrowserKind::EdgeBeta => &["microsoft-edge-beta"],
        BrowserKind::EdgeCanary => &["microsoft-edge-dev"],
        BrowserKind::Brave => &["brave"],
        BrowserKind::BraveBeta => &["brave-beta"],
        BrowserKind::BraveNightly => &["brave-nightly"],
        BrowserKind::Chromium => &["chromium"],
        BrowserKind::Vivaldi => &["vivaldi"],
        BrowserKind::Yandex => &["yandex-browser"],
        _ => &[],
    }
}

#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
fn linux_zen_flatpak_app_ids() -> &'static [&'static str] {
    &["app.zen_browser.zen", "io.github.zen_browser.zen"]
}

#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
fn linux_zen_snap_names() -> &'static [&'static str] {
    &["zen-browser", "zen"]
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
        let _folder = folder;
        let roaming = windows_roaming_app_data(home);
        match browser {
            BrowserKind::Zen => windows_local_app_data(home).join("zen").join("Profiles"),
            _ => roaming.join("Mozilla").join("Firefox").join("Profiles"),
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _folder = folder;
        match browser {
            BrowserKind::Firefox => home.join(".mozilla").join("firefox"),
            BrowserKind::Zen => home.join(".zen"),
            _ => home.join(".mozilla").join("firefox"),
        }
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
pub(crate) fn gecko_test_profile_dir(
    home: &Path,
    browser: BrowserKind,
    profile_folder: &str,
) -> PathBuf {
    gecko_profiles_root(home, browser)
        .unwrap_or_else(|| panic!("gecko root missing for {browser:?}"))
        .join(profile_folder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
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
    #[cfg(all(unix, not(target_os = "macos")))]
    fn linux_chromium_family_roots_cover_modern_browsers() {
        let home = Path::new("/home/test");

        assert_eq!(
            chromium_user_data_root_for(home, BrowserKind::Edge),
            PathBuf::from("/home/test/.config/microsoft-edge")
        );
        assert_eq!(
            chromium_user_data_root_for(home, BrowserKind::Brave),
            PathBuf::from("/home/test/.config/BraveSoftware/Brave-Browser")
        );
        assert_eq!(
            chromium_user_data_root_for(home, BrowserKind::Chromium),
            PathBuf::from("/home/test/.config/chromium")
        );
        assert_eq!(
            chromium_user_data_root_for(home, BrowserKind::Vivaldi),
            PathBuf::from("/home/test/.config/vivaldi")
        );
        assert_eq!(
            chromium_user_data_root_for(home, BrowserKind::ChromeCanary),
            PathBuf::from("/home/test/.config/google-chrome-unstable")
        );
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
    #[cfg(target_os = "macos")]
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
    fn linux_gecko_profiles_roots_are_profile_containers() {
        let home = Path::new("/home/test");
        assert_eq!(
            gecko_profiles_root(home, BrowserKind::Firefox).expect("firefox root"),
            PathBuf::from("/home/test/.mozilla/firefox")
        );
        assert_eq!(
            gecko_profiles_root(home, BrowserKind::Zen).expect("zen root"),
            PathBuf::from("/home/test/.zen")
        );
    }

    #[test]
    fn zen_flatpak_profiles_root_uses_flathub_app_storage() {
        let home = Path::new("/home/test");

        assert_eq!(
            linux_flatpak_home_relative_root(home, "app.zen_browser.zen", ".zen"),
            PathBuf::from("/home/test/.var/app/app.zen_browser.zen/.zen")
        );
    }

    #[test]
    fn linux_flatpak_config_root_uses_app_private_config_storage() {
        let home = Path::new("/home/test");

        assert_eq!(
            linux_flatpak_config_root(home, "com.vivaldi.Vivaldi", "vivaldi"),
            PathBuf::from("/home/test/.var/app/com.vivaldi.Vivaldi/config/vivaldi")
        );
    }

    #[test]
    fn linux_flatpak_home_relative_root_preserves_gecko_dot_directories() {
        let home = Path::new("/home/test");

        assert_eq!(
            linux_flatpak_home_relative_root(home, "org.mozilla.firefox", ".mozilla/firefox"),
            PathBuf::from("/home/test/.var/app/org.mozilla.firefox/.mozilla/firefox")
        );
    }

    #[test]
    fn linux_snap_common_config_root_uses_snap_common_storage() {
        let home = Path::new("/home/test");

        assert_eq!(
            linux_snap_common_config_root(home, "chromium", "chromium"),
            PathBuf::from("/home/test/snap/chromium/common/.config/chromium")
        );
    }

    #[test]
    fn linux_flatpak_app_ids_cover_supported_chromium_family_packages() {
        assert_eq!(
            linux_flatpak_app_ids(BrowserKind::Chrome),
            &["com.google.Chrome"]
        );
        assert_eq!(
            linux_flatpak_app_ids(BrowserKind::ChromeBeta),
            &["com.google.Chrome.Beta"]
        );
        assert_eq!(
            linux_flatpak_app_ids(BrowserKind::ChromeCanary),
            &["com.google.ChromeDev"]
        );
        assert_eq!(
            linux_flatpak_app_ids(BrowserKind::Edge),
            &["com.microsoft.Edge"]
        );
        assert_eq!(
            linux_flatpak_app_ids(BrowserKind::Brave),
            &["com.brave.Browser"]
        );
        assert_eq!(
            linux_flatpak_app_ids(BrowserKind::Chromium),
            &["org.chromium.Chromium"]
        );
        assert_eq!(
            linux_flatpak_app_ids(BrowserKind::Vivaldi),
            &["com.vivaldi.Vivaldi"]
        );
        assert_eq!(
            linux_flatpak_app_ids(BrowserKind::Yandex),
            &["ru.yandex.Browser"]
        );
    }

    #[test]
    fn linux_snap_names_cover_supported_chromium_family_packages() {
        assert_eq!(linux_snap_names(BrowserKind::Chrome), &["google-chrome"]);
        assert_eq!(linux_snap_names(BrowserKind::Edge), &["microsoft-edge"]);
        assert_eq!(linux_snap_names(BrowserKind::Brave), &["brave"]);
        assert_eq!(linux_snap_names(BrowserKind::Chromium), &["chromium"]);
        assert_eq!(linux_snap_names(BrowserKind::Vivaldi), &["vivaldi"]);
        assert_eq!(linux_snap_names(BrowserKind::Yandex), &["yandex-browser"]);
    }

    #[test]
    fn linux_zen_flatpak_app_ids_include_current_and_historical_ids() {
        assert_eq!(
            linux_zen_flatpak_app_ids(),
            &["app.zen_browser.zen", "io.github.zen_browser.zen"]
        );
    }

    #[test]
    #[cfg(all(unix, not(target_os = "macos")))]
    fn linux_supported_chromium_roots_include_native_flatpak_and_snap_layouts() {
        let home = Path::new("/home/test");
        let roots = chromium_user_data_roots(home, BrowserKind::Vivaldi);

        assert!(roots.contains(&PathBuf::from("/home/test/.config/vivaldi")));
        assert!(roots.contains(&PathBuf::from(
            "/home/test/.var/app/com.vivaldi.Vivaldi/config/vivaldi"
        )));
        assert!(roots.contains(&PathBuf::from(
            "/home/test/snap/vivaldi/common/.config/vivaldi"
        )));
    }

    #[test]
    #[cfg(all(unix, not(target_os = "macos")))]
    fn linux_supported_gecko_roots_include_firefox_and_zen_package_layouts() {
        let home = Path::new("/home/test");

        let firefox_roots = gecko_profiles_roots(home, BrowserKind::Firefox);
        assert!(firefox_roots.contains(&PathBuf::from("/home/test/.mozilla/firefox")));
        assert!(firefox_roots.contains(&PathBuf::from(
            "/home/test/.var/app/org.mozilla.firefox/.mozilla/firefox"
        )));
        assert!(firefox_roots.contains(&PathBuf::from(
            "/home/test/.var/app/org.mozilla.firefox/config/mozilla/firefox"
        )));
        assert!(firefox_roots.contains(&PathBuf::from(
            "/home/test/.var/app/org.mozilla.firefox/.config/mozilla/firefox"
        )));
        assert!(firefox_roots.contains(&PathBuf::from(
            "/home/test/snap/firefox/common/.mozilla/firefox"
        )));

        let zen_roots = gecko_profiles_roots(home, BrowserKind::Zen);
        assert!(zen_roots.contains(&PathBuf::from("/home/test/.zen")));
        assert!(zen_roots.contains(&PathBuf::from(
            "/home/test/.var/app/app.zen_browser.zen/.zen"
        )));
        assert!(zen_roots.contains(&PathBuf::from(
            "/home/test/.var/app/io.github.zen_browser.zen/.zen"
        )));
        assert!(zen_roots.contains(&PathBuf::from(
            "/home/test/.var/app/app.zen_browser.zen/config/zen"
        )));
        assert!(zen_roots.contains(&PathBuf::from(
            "/home/test/.var/app/app.zen_browser.zen/.config/zen"
        )));
        assert!(zen_roots.contains(&PathBuf::from("/home/test/snap/zen-browser/common/.zen")));
        assert!(zen_roots.contains(&PathBuf::from(
            "/home/test/snap/zen-browser/common/.config/zen"
        )));
    }
}
