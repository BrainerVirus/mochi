#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoratedWindowCreationMode {
    StartupPrecreate,
    OnDemand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoratedWindowInitialVisibility {
    Hidden,
    Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstShowSequence {
    ShowUnminimizeFocus,
    AlreadyVisibleFocus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecoratedWindowPolicy {
    pub name: &'static str,
    pub creation_mode: DecoratedWindowCreationMode,
    pub initial_visibility: DecoratedWindowInitialVisibility,
    pub first_show_sequence: FirstShowSequence,
    pub mutate_size_before_first_show: bool,
}

impl DecoratedWindowPolicy {
    pub fn for_target_os(target_os: &str) -> Self {
        match target_os {
            "linux" => Self {
                name: "linux-on-demand-visible",
                creation_mode: DecoratedWindowCreationMode::OnDemand,
                initial_visibility: DecoratedWindowInitialVisibility::Visible,
                first_show_sequence: FirstShowSequence::AlreadyVisibleFocus,
                mutate_size_before_first_show: false,
            },
            _ => Self {
                name: "startup-hidden",
                creation_mode: DecoratedWindowCreationMode::StartupPrecreate,
                initial_visibility: DecoratedWindowInitialVisibility::Hidden,
                first_show_sequence: FirstShowSequence::ShowUnminimizeFocus,
                mutate_size_before_first_show: true,
            },
        }
    }

    pub const fn creation_label(self) -> &'static str {
        match self.creation_mode {
            DecoratedWindowCreationMode::StartupPrecreate => "startup-precreate",
            DecoratedWindowCreationMode::OnDemand => "on-demand",
        }
    }

    pub const fn initial_visibility_label(self) -> &'static str {
        match self.initial_visibility {
            DecoratedWindowInitialVisibility::Hidden => "hidden",
            DecoratedWindowInitialVisibility::Visible => "visible",
        }
    }
}

pub fn active_decorated_window_policy() -> DecoratedWindowPolicy {
    DecoratedWindowPolicy::for_target_os(std::env::consts::OS)
}

pub fn should_precreate_decorated_windows_at_startup() -> bool {
    active_decorated_window_policy().creation_mode == DecoratedWindowCreationMode::StartupPrecreate
}

pub fn decorated_window_initial_visibility() -> DecoratedWindowInitialVisibility {
    active_decorated_window_policy().initial_visibility
}

pub fn should_mutate_size_before_first_show() -> bool {
    active_decorated_window_policy().mutate_size_before_first_show
}

pub fn first_show_sequence() -> FirstShowSequence {
    active_decorated_window_policy().first_show_sequence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_policy_uses_on_demand_visible_decorated_windows() {
        let policy = DecoratedWindowPolicy::for_target_os("linux");

        assert_eq!(policy.name, "linux-on-demand-visible");
        assert_eq!(policy.creation_mode, DecoratedWindowCreationMode::OnDemand);
        assert_eq!(
            policy.initial_visibility,
            DecoratedWindowInitialVisibility::Visible
        );
        assert_eq!(
            policy.first_show_sequence,
            FirstShowSequence::AlreadyVisibleFocus
        );
        assert!(!policy.mutate_size_before_first_show);
    }

    #[test]
    fn macos_and_windows_keep_startup_hidden_policy() {
        for os in ["macos", "windows"] {
            let policy = DecoratedWindowPolicy::for_target_os(os);

            assert_eq!(policy.name, "startup-hidden");
            assert_eq!(
                policy.creation_mode,
                DecoratedWindowCreationMode::StartupPrecreate
            );
            assert_eq!(
                policy.initial_visibility,
                DecoratedWindowInitialVisibility::Hidden
            );
            assert_eq!(
                policy.first_show_sequence,
                FirstShowSequence::ShowUnminimizeFocus
            );
            assert!(policy.mutate_size_before_first_show);
        }
    }

    #[test]
    fn unknown_targets_use_startup_hidden_policy() {
        let policy = DecoratedWindowPolicy::for_target_os("freebsd");

        assert_eq!(policy.name, "startup-hidden");
        assert_eq!(
            policy.creation_mode,
            DecoratedWindowCreationMode::StartupPrecreate
        );
    }
}
