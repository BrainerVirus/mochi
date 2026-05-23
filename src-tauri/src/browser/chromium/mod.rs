#[cfg(target_os = "macos")]
mod decrypt;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod stub;

#[cfg(target_os = "macos")]
pub use macos::{chromium_decryption_key, discover_chromium_stores, read_chromium_cookies};
#[cfg(not(target_os = "macos"))]
pub use stub::{
    chromium_decryption_key, discover_chromium_stores, read_chromium_cookies, ChromiumCookieStore,
};
