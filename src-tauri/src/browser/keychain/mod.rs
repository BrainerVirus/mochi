#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod stub;

#[cfg(target_os = "macos")]
pub use macos::read_safe_storage_password;
#[cfg(not(target_os = "macos"))]
pub use stub::read_safe_storage_password;
