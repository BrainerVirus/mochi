use crate::browser::catalog::BrowserKind;

pub fn read_safe_storage_password(_browser: BrowserKind) -> Result<String, String> {
    Err("Chromium Keychain access is only supported on macOS".into())
}
