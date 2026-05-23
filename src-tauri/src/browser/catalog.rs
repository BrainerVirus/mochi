//! Browser catalog and import order.
//!
//! Derived from CodexBar / SweetCookieKit (MIT): `BrowserCatalog.swift`,
//! `ProviderBrowserCookieDefaults.cursorCookieImportOrder`.

/// Supported browsers for cookie import on macOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserKind {
    Safari,
    Chrome,
    Edge,
    Brave,
    Arc,
    Dia,
    ChatGptAtlas,
    Chromium,
    Helium,
    Vivaldi,
    Yandex,
    Firefox,
    Zen,
    ChromeBeta,
    ChromeCanary,
    ArcBeta,
    ArcCanary,
    BraveBeta,
    BraveNightly,
    EdgeBeta,
    EdgeCanary,
    Comet,
}

impl BrowserKind {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Safari => "Safari",
            Self::Chrome => "Chrome",
            Self::Edge => "Microsoft Edge",
            Self::Brave => "Brave",
            Self::Arc => "Arc",
            Self::Dia => "Dia",
            Self::ChatGptAtlas => "ChatGPT Atlas",
            Self::Chromium => "Chromium",
            Self::Helium => "Helium",
            Self::Vivaldi => "Vivaldi",
            Self::Yandex => "Yandex Browser",
            Self::Firefox => "Firefox",
            Self::Zen => "Zen",
            Self::ChromeBeta => "Chrome Beta",
            Self::ChromeCanary => "Chrome Canary",
            Self::ArcBeta => "Arc Beta",
            Self::ArcCanary => "Arc Canary",
            Self::BraveBeta => "Brave Beta",
            Self::BraveNightly => "Brave Nightly",
            Self::EdgeBeta => "Microsoft Edge Beta",
            Self::EdgeCanary => "Microsoft Edge Canary",
            Self::Comet => "Comet",
        }
    }

    pub fn engine(self) -> BrowserEngine {
        match self {
            Self::Safari => BrowserEngine::WebKit,
            Self::Firefox | Self::Zen => BrowserEngine::Gecko,
            _ => BrowserEngine::Chromium,
        }
    }

    /// Chromium profile root under `~/Library/Application Support/`.
    pub fn chromium_support_path(self) -> Option<&'static str> {
        match self {
            Self::Chrome => Some("Google/Chrome"),
            Self::ChromeBeta => Some("Google/Chrome Beta"),
            Self::ChromeCanary => Some("Google/Chrome Canary"),
            Self::Edge => Some("Microsoft Edge"),
            Self::EdgeBeta => Some("Microsoft Edge Beta"),
            Self::EdgeCanary => Some("Microsoft Edge Canary"),
            Self::Brave => Some("BraveSoftware/Brave-Browser"),
            Self::BraveBeta => Some("BraveSoftware/Brave-Browser-Beta"),
            Self::BraveNightly => Some("BraveSoftware/Brave-Browser-Nightly"),
            Self::Arc => Some("Arc/User Data"),
            Self::ArcBeta => Some("Arc Beta/User Data"),
            Self::ArcCanary => Some("Arc Canary/User Data"),
            Self::Dia => Some("Dia/User Data"),
            Self::ChatGptAtlas => Some("com.openai.atlas/browser-data/host"),
            Self::Chromium => Some("Chromium"),
            Self::Helium => Some("net.imput.helium"),
            Self::Vivaldi => Some("Vivaldi"),
            Self::Yandex => Some("Yandex/YandexBrowser"),
            Self::Comet => Some("Comet"),
            Self::Safari | Self::Firefox | Self::Zen => None,
        }
    }

    /// Gecko profiles folder under `~/Library/Application Support/`.
    pub fn gecko_profiles_folder(self) -> Option<&'static str> {
        match self {
            Self::Firefox => Some("Firefox"),
            Self::Zen => Some("zen"),
            _ => None,
        }
    }

    /// macOS Keychain labels for Chromium cookie decryption.
    pub fn safe_storage_labels(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Chrome => &[("Chrome Safe Storage", "Chrome")],
            Self::Chromium => &[("Chromium Safe Storage", "Chromium")],
            Self::Brave => &[("Brave Safe Storage", "Brave")],
            Self::Arc => &[("Arc Safe Storage", "Arc")],
            Self::ArcBeta => &[("Arc Safe Storage", "Arc Beta")],
            Self::ArcCanary => &[("Arc Safe Storage", "Arc Canary")],
            Self::Edge => &[("Microsoft Edge Safe Storage", "Microsoft Edge")],
            Self::Dia => &[("Dia Safe Storage", "Dia")],
            Self::ChatGptAtlas => &[
                ("ChatGPT Atlas Safe Storage", "ChatGPT Atlas"),
                ("ChatGPT Atlas Safe Storage", "com.openai.atlas"),
                ("com.openai.atlas Safe Storage", "com.openai.atlas"),
            ],
            Self::Helium => &[
                ("Helium Storage Key", "Helium"),
                ("Helium Safe Storage", "Helium"),
                ("net.imput.helium Safe Storage", "net.imput.helium"),
            ],
            Self::Vivaldi => &[("Vivaldi Safe Storage", "Vivaldi")],
            Self::Yandex => &[("Yandex Safe Storage", "Yandex")],
            Self::Comet => &[("Comet Safe Storage", "Comet")],
            _ => &[],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserEngine {
    WebKit,
    Chromium,
    Gecko,
}

/// Default browser import order from SweetCookieKit `BrowserCatalog.defaultImportOrder`.
pub fn default_import_order() -> Vec<BrowserKind> {
    vec![
        BrowserKind::Safari,
        BrowserKind::Chrome,
        BrowserKind::Edge,
        BrowserKind::Brave,
        BrowserKind::Arc,
        BrowserKind::Dia,
        BrowserKind::ChatGptAtlas,
        BrowserKind::Chromium,
        BrowserKind::Helium,
        BrowserKind::Vivaldi,
        BrowserKind::Yandex,
        BrowserKind::Firefox,
        BrowserKind::Zen,
        BrowserKind::ChromeBeta,
        BrowserKind::ChromeCanary,
        BrowserKind::ArcBeta,
        BrowserKind::ArcCanary,
        BrowserKind::BraveBeta,
        BrowserKind::BraveNightly,
        BrowserKind::EdgeBeta,
        BrowserKind::EdgeCanary,
        BrowserKind::Comet,
    ]
}

/// Cursor prefers Safari first, then the default order (CodexBar `cursorCookieImportOrder`).
pub fn cursor_import_order() -> Vec<BrowserKind> {
    let mut order = vec![BrowserKind::Safari];
    order.extend(
        default_import_order()
            .into_iter()
            .filter(|browser| *browser != BrowserKind::Safari),
    );
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_import_order_puts_safari_first() {
        let order = cursor_import_order();
        assert_eq!(order.first(), Some(&BrowserKind::Safari));
        assert!(order.contains(&BrowserKind::Zen));
        assert!(order.contains(&BrowserKind::Firefox));
    }

    #[test]
    fn zen_uses_gecko_profiles_under_zen_folder() {
        assert_eq!(BrowserKind::Zen.gecko_profiles_folder(), Some("zen"));
        assert_eq!(BrowserKind::Zen.engine(), BrowserEngine::Gecko);
    }
}
