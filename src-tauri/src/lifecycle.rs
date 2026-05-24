use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct AppLifecycle {
    explicit_quit_requested: AtomicBool,
}

impl AppLifecycle {
    pub fn request_quit(&self) {
        self.explicit_quit_requested.store(true, Ordering::SeqCst);
    }

    pub fn should_prevent_exit(&self) -> bool {
        !self.explicit_quit_requested.load(Ordering::SeqCst)
    }
}

pub fn should_prevent_exit_request(lifecycle: Option<&AppLifecycle>) -> bool {
    lifecycle.is_none_or(AppLifecycle::should_prevent_exit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implicit_exit_requests_are_prevented_for_tray_app() {
        let lifecycle = AppLifecycle::default();

        assert!(lifecycle.should_prevent_exit());
    }

    #[test]
    fn explicit_quit_request_allows_exit() {
        let lifecycle = AppLifecycle::default();

        lifecycle.request_quit();

        assert!(!lifecycle.should_prevent_exit());
    }

    #[test]
    fn missing_lifecycle_state_prevents_exit_conservatively() {
        assert!(should_prevent_exit_request(None));
    }

    #[test]
    fn explicit_lifecycle_quit_allows_exit_request() {
        let lifecycle = AppLifecycle::default();

        lifecycle.request_quit();

        assert!(!should_prevent_exit_request(Some(&lifecycle)));
    }
}
