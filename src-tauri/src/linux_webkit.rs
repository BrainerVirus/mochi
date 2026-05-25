pub const WEBKIT_DMABUF_ENV: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";
pub const WEBKIT_COMPOSITING_ENV: &str = "WEBKIT_DISABLE_COMPOSITING_MODE";
pub const WEBKIT_ACCELERATION_ESCAPE_ENV: &str = "MOCHI_ALLOW_WEBKIT_ACCELERATION";

const WEBKIT_WORKAROUND_VALUE: &str = "1";

pub fn linux_webkit_env_defaults<F>(get_env: F) -> Vec<(&'static str, &'static str)>
where
    F: Fn(&str) -> Option<String>,
{
    if get_env(WEBKIT_ACCELERATION_ESCAPE_ENV).is_some_and(|value| value == "1") {
        return Vec::new();
    }

    [WEBKIT_DMABUF_ENV, WEBKIT_COMPOSITING_ENV]
        .into_iter()
        .filter(|key| get_env(key).is_none())
        .map(|key| (key, WEBKIT_WORKAROUND_VALUE))
        .collect()
}

#[cfg(target_os = "linux")]
pub fn apply_linux_webkit_workarounds() {
    for (key, value) in linux_webkit_env_defaults(|key| {
        std::env::var_os(key).map(|value| value.to_string_lossy().into_owned())
    }) {
        std::env::set_var(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_webkit_defaults_are_applied_when_missing() {
        let defaults = linux_webkit_env_defaults(|key| match key {
            WEBKIT_DMABUF_ENV | WEBKIT_COMPOSITING_ENV => None,
            WEBKIT_ACCELERATION_ESCAPE_ENV => None,
            _ => unreachable!("unexpected env key {key}"),
        });

        assert_eq!(
            defaults,
            vec![(WEBKIT_DMABUF_ENV, "1"), (WEBKIT_COMPOSITING_ENV, "1"),]
        );
    }

    #[test]
    fn linux_webkit_defaults_preserve_existing_values() {
        let defaults = linux_webkit_env_defaults(|key| match key {
            WEBKIT_DMABUF_ENV => Some("0".to_string()),
            WEBKIT_COMPOSITING_ENV => Some("custom".to_string()),
            WEBKIT_ACCELERATION_ESCAPE_ENV => None,
            _ => unreachable!("unexpected env key {key}"),
        });

        assert!(defaults.is_empty());
    }

    #[test]
    fn linux_webkit_defaults_can_be_disabled() {
        let defaults = linux_webkit_env_defaults(|key| match key {
            WEBKIT_ACCELERATION_ESCAPE_ENV => Some("1".to_string()),
            WEBKIT_DMABUF_ENV | WEBKIT_COMPOSITING_ENV => None,
            _ => unreachable!("unexpected env key {key}"),
        });

        assert!(defaults.is_empty());
    }
}
