#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxWindowExperiment {
    BaselineSequencedLogs,
    OnDemandVisible,
    OnDemandHidden,
    BuilderSizeOnly,
    ShowFocusOnly,
    UnminimizeShowFocus,
}

impl LinuxWindowExperiment {
    pub const fn name(self) -> &'static str {
        match self {
            Self::BaselineSequencedLogs => "baseline-sequenced-logs",
            Self::OnDemandVisible => "on-demand-visible",
            Self::OnDemandHidden => "on-demand-hidden",
            Self::BuilderSizeOnly => "builder-size-only",
            Self::ShowFocusOnly => "show-focus-only",
            Self::UnminimizeShowFocus => "unminimize-show-focus",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "on-demand-visible" => Self::OnDemandVisible,
            "on-demand-hidden" => Self::OnDemandHidden,
            "builder-size-only" => Self::BuilderSizeOnly,
            "show-focus-only" => Self::ShowFocusOnly,
            "unminimize-show-focus" => Self::UnminimizeShowFocus,
            "baseline-sequenced-logs" => Self::BaselineSequencedLogs,
            _ => Self::BaselineSequencedLogs,
        }
    }
}

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
    ShowFocus,
    UnminimizeShowFocus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinuxWindowPolicy {
    pub experiment: LinuxWindowExperiment,
    pub creation_mode: DecoratedWindowCreationMode,
    pub initial_visibility: DecoratedWindowInitialVisibility,
    pub first_show_sequence: FirstShowSequence,
    pub mutate_size_before_first_show: bool,
}

impl LinuxWindowPolicy {
    pub const fn for_experiment(experiment: LinuxWindowExperiment) -> Self {
        match experiment {
            LinuxWindowExperiment::BaselineSequencedLogs => Self {
                experiment,
                creation_mode: DecoratedWindowCreationMode::StartupPrecreate,
                initial_visibility: DecoratedWindowInitialVisibility::Hidden,
                first_show_sequence: FirstShowSequence::ShowUnminimizeFocus,
                mutate_size_before_first_show: true,
            },
            LinuxWindowExperiment::OnDemandVisible => Self {
                experiment,
                creation_mode: DecoratedWindowCreationMode::OnDemand,
                initial_visibility: DecoratedWindowInitialVisibility::Visible,
                first_show_sequence: FirstShowSequence::AlreadyVisibleFocus,
                mutate_size_before_first_show: false,
            },
            LinuxWindowExperiment::OnDemandHidden => Self {
                experiment,
                creation_mode: DecoratedWindowCreationMode::OnDemand,
                initial_visibility: DecoratedWindowInitialVisibility::Hidden,
                first_show_sequence: FirstShowSequence::ShowUnminimizeFocus,
                mutate_size_before_first_show: false,
            },
            LinuxWindowExperiment::BuilderSizeOnly => Self {
                experiment,
                creation_mode: DecoratedWindowCreationMode::OnDemand,
                initial_visibility: DecoratedWindowInitialVisibility::Visible,
                first_show_sequence: FirstShowSequence::AlreadyVisibleFocus,
                mutate_size_before_first_show: false,
            },
            LinuxWindowExperiment::ShowFocusOnly => Self {
                experiment,
                creation_mode: DecoratedWindowCreationMode::OnDemand,
                initial_visibility: DecoratedWindowInitialVisibility::Hidden,
                first_show_sequence: FirstShowSequence::ShowFocus,
                mutate_size_before_first_show: false,
            },
            LinuxWindowExperiment::UnminimizeShowFocus => Self {
                experiment,
                creation_mode: DecoratedWindowCreationMode::OnDemand,
                initial_visibility: DecoratedWindowInitialVisibility::Hidden,
                first_show_sequence: FirstShowSequence::UnminimizeShowFocus,
                mutate_size_before_first_show: false,
            },
        }
    }
}

pub fn active_linux_window_experiment() -> LinuxWindowExperiment {
    LinuxWindowExperiment::parse(option_env!("MOCHI_LINUX_WINDOW_EXPERIMENT").unwrap_or(
        "baseline-sequenced-logs",
    ))
}

pub fn active_linux_window_policy() -> LinuxWindowPolicy {
    LinuxWindowPolicy::for_experiment(active_linux_window_experiment())
}

pub fn should_precreate_decorated_windows_at_startup() -> bool {
    if cfg!(target_os = "linux") {
        active_linux_window_policy().creation_mode == DecoratedWindowCreationMode::StartupPrecreate
    } else {
        true
    }
}

pub fn decorated_window_initial_visibility() -> DecoratedWindowInitialVisibility {
    if cfg!(target_os = "linux") {
        active_linux_window_policy().initial_visibility
    } else {
        DecoratedWindowInitialVisibility::Hidden
    }
}

pub fn should_mutate_size_before_first_show() -> bool {
    if cfg!(target_os = "linux") {
        active_linux_window_policy().mutate_size_before_first_show
    } else {
        true
    }
}

pub fn first_show_sequence() -> FirstShowSequence {
    if cfg!(target_os = "linux") {
        active_linux_window_policy().first_show_sequence
    } else {
        FirstShowSequence::ShowUnminimizeFocus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_linux_window_experiments() {
        assert_eq!(
            LinuxWindowExperiment::parse("baseline-sequenced-logs"),
            LinuxWindowExperiment::BaselineSequencedLogs
        );
        assert_eq!(
            LinuxWindowExperiment::parse("on-demand-visible"),
            LinuxWindowExperiment::OnDemandVisible
        );
        assert_eq!(
            LinuxWindowExperiment::parse("on-demand-hidden"),
            LinuxWindowExperiment::OnDemandHidden
        );
        assert_eq!(
            LinuxWindowExperiment::parse("builder-size-only"),
            LinuxWindowExperiment::BuilderSizeOnly
        );
        assert_eq!(
            LinuxWindowExperiment::parse("show-focus-only"),
            LinuxWindowExperiment::ShowFocusOnly
        );
        assert_eq!(
            LinuxWindowExperiment::parse("unminimize-show-focus"),
            LinuxWindowExperiment::UnminimizeShowFocus
        );
    }

    #[test]
    fn unknown_experiment_falls_back_to_baseline() {
        assert_eq!(
            LinuxWindowExperiment::parse("not-real"),
            LinuxWindowExperiment::BaselineSequencedLogs
        );
    }

    #[test]
    fn on_demand_visible_policy_creates_decorated_windows_on_user_action() {
        let policy = LinuxWindowPolicy::for_experiment(LinuxWindowExperiment::OnDemandVisible);
        assert_eq!(policy.creation_mode, DecoratedWindowCreationMode::OnDemand);
        assert_eq!(policy.initial_visibility, DecoratedWindowInitialVisibility::Visible);
        assert_eq!(policy.first_show_sequence, FirstShowSequence::AlreadyVisibleFocus);
        assert!(!policy.mutate_size_before_first_show);
    }

    #[test]
    fn baseline_policy_matches_current_startup_hidden_behavior() {
        let policy = LinuxWindowPolicy::for_experiment(LinuxWindowExperiment::BaselineSequencedLogs);
        assert_eq!(policy.creation_mode, DecoratedWindowCreationMode::StartupPrecreate);
        assert_eq!(policy.initial_visibility, DecoratedWindowInitialVisibility::Hidden);
        assert_eq!(policy.first_show_sequence, FirstShowSequence::ShowUnminimizeFocus);
        assert!(policy.mutate_size_before_first_show);
    }
}
