#[cfg(all(unix, not(target_os = "macos")))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use std::path::Path;

use crate::browser::catalog::BrowserKind;

pub fn read_safe_storage_password(home: &Path, browser: BrowserKind) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        macos::read_safe_storage_password(home, browser)
    }
    #[cfg(target_os = "windows")]
    {
        windows::read_safe_storage_password(home, browser)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        linux::read_safe_storage_password(home, browser)
    }
}
