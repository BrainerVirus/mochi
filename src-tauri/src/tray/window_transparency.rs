//! Whether tray/settings webviews use OS-backed transparency (vibrancy / Mica).

/// macOS and Windows apply native window effects; Linux uses opaque windows + CSS surfaces.
pub fn window_uses_native_transparency() -> bool {
    cfg!(any(target_os = "macos", target_os = "windows"))
}

#[cfg(test)]
mod tests {
    use super::window_uses_native_transparency;

    #[test]
    fn native_transparency_matches_compile_time_target() {
        let expected = cfg!(any(target_os = "macos", target_os = "windows"));
        assert_eq!(window_uses_native_transparency(), expected);
    }
}
