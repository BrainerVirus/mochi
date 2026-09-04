//! Usage dashboard TUI: read-only provider usage table.
//!
//! Percentages and window labels only — snapshot messages (which can carry
//! fetch errors) and secrets are never rendered.

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame, Terminal,
};

use super::{install_panic_hook, TuiGuard};
use crate::core::usage_state::ProviderUsageState;
use crate::tray::provider_display_name;

/// Display-only percent: explicit 0..=100 clamp so out-of-range values
/// saturate instead of wrapping on the `as u8` cast. (`UsageWindow::new`
/// already clamps at construction; this pins the display contract for
/// windows built or deserialized by other paths.)
pub fn format_used_percent(used_percent: f32) -> String {
    format!("{}%", used_percent.round().clamp(0.0, 100.0) as u8)
}

/// First-line, length-capped cause for the refresh-failed indicator.
pub fn brief_refresh_error(message: &str) -> String {
    message
        .lines()
        .next()
        .unwrap_or("unknown error")
        .trim()
        .chars()
        .take(120)
        .collect()
}

/// Shared refresh fold: success swaps in fresh data and clears the error;
/// failure keeps the stale data and records the brief cause.
pub fn apply_refresh<T>(
    current: T,
    result: anyhow::Result<T>,
    refresh_error: &mut Option<String>,
) -> T {
    match result {
        Ok(fresh) => {
            *refresh_error = None;
            fresh
        }
        Err(error) => {
            *refresh_error = Some(brief_refresh_error(&error.to_string()));
            current
        }
    }
}

/// Unit-testable key decision for the dashboard loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardKeyAction {
    Quit,
    Refresh,
    Ignore,
}

/// Full key-event entry point: Ctrl+C quits from anywhere
/// (the `KeyCode`-only path cannot see modifiers).
pub fn handle_dashboard_key_event(key: KeyEvent) -> DashboardKeyAction {
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
    {
        return DashboardKeyAction::Quit;
    }
    handle_dashboard_key(key.code)
}

pub fn handle_dashboard_key(code: KeyCode) -> DashboardKeyAction {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => DashboardKeyAction::Quit,
        KeyCode::Char('r') | KeyCode::Char('R') => DashboardKeyAction::Refresh,
        _ => DashboardKeyAction::Ignore,
    }
}

/// Pure render over `&[ProviderUsageState]`; TestBackend-compatible (no terminal I/O).
/// `refresh_error` (when `Some`) renders a visible failure indicator while the
/// stale table stays on screen.
pub fn render_usage_dashboard(
    frame: &mut Frame,
    states: &[ProviderUsageState],
    refresh_error: Option<&str>,
) {
    let area = frame.area();
    let failed = refresh_error.map(str::trim).filter(|s| !s.is_empty());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(match failed {
            Some(_) => [Constraint::Min(1), Constraint::Length(2)],
            None => [Constraint::Min(1), Constraint::Length(1)],
        })
        .split(area);
    let mut rows = Vec::new();
    if states.is_empty() {
        rows.push(Row::new(vec![
            Cell::from("No usage data cached."),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ]));
    }
    for state in states {
        let Some(snapshot) = state.snapshot.as_ref() else {
            rows.push(Row::new(vec![
                Cell::from(provider_display_name(state.provider)),
                Cell::from("no data"),
                Cell::from(""),
                Cell::from(""),
            ]));
            continue;
        };
        for window in snapshot.rate_windows() {
            rows.push(Row::new(vec![
                Cell::from(provider_display_name(snapshot.provider)),
                Cell::from(window.label.clone()),
                Cell::from(format_used_percent(window.used_percent)),
                Cell::from(window.resets_at.clone().unwrap_or_default()),
            ]));
        }
    }
    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(9),
            Constraint::Length(5),
            Constraint::Min(8),
        ],
    )
    .header(Row::new(["Provider", "Window", "Used", "Reset"]))
    .block(Block::default().title("Mochi usage").borders(Borders::ALL));
    frame.render_widget(table, chunks[0]);
    let footer = match failed {
        Some(cause) => {
            format!("q/Esc quit · r refresh\n(refresh failed: {cause} — showing cached data)")
        }
        None => "q/Esc quit · r refresh".to_string(),
    };
    frame.render_widget(Paragraph::new(footer), chunks[1]);
}

/// Run the fullscreen dashboard event loop. Read-only: q/Esc exits and `r`
/// re-reads the cached store (no live fetch, no mutations).
pub fn run_usage_dashboard(provider: Option<&str>, refresh: bool) -> anyhow::Result<()> {
    let mut states = crate::cli_usage_states(provider, refresh)?;
    let mut refresh_error: Option<String> = None;
    // Install LAST so terminal restore runs before the diagnostics logger.
    install_panic_hook();
    {
        let _guard = TuiGuard::enter()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        loop {
            terminal
                .draw(|frame| render_usage_dashboard(frame, &states, refresh_error.as_deref()))?;
            if event::poll(Duration::from_millis(250))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        match handle_dashboard_key_event(key) {
                            DashboardKeyAction::Quit => break,
                            DashboardKeyAction::Refresh => {
                                states = apply_refresh(
                                    states,
                                    crate::cli_usage_states(provider, false),
                                    &mut refresh_error,
                                );
                            }
                            DashboardKeyAction::Ignore => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use ratatui::{backend::TestBackend, Terminal};

    use super::{
        apply_refresh, brief_refresh_error, format_used_percent, handle_dashboard_key,
        handle_dashboard_key_event, render_usage_dashboard, DashboardKeyAction,
    };
    use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
    use crate::core::usage_state::ProviderUsageState;

    fn fixture_states() -> Vec<ProviderUsageState> {
        let snapshot = UsageSnapshot::new(
            ProviderId::Claude,
            UsageWindow::new("5 hours", 64.0, None),
            Some(UsageWindow::new("Weekly", 10.0, None)),
            "2026-06-04T12:00:00Z",
            "test",
        );
        vec![ProviderUsageState::fresh(snapshot)]
    }

    fn render_text(states: &[ProviderUsageState], refresh_error: Option<&str>) -> String {
        // 80 wide: fits the two-line refresh-failed footer without truncation.
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_usage_dashboard(frame, states, refresh_error))
            .expect("draw");
        terminal.backend().to_string()
    }

    fn ctrl_c() -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn dashboard_uses_widget_labels() {
        let content = render_text(&fixture_states(), None);
        assert!(content.contains("Weekly"));
        assert!(content.contains("5 hours"));
    }

    #[test]
    fn dashboard_renders_no_secrets() {
        let mut states = fixture_states();
        states.push(ProviderUsageState::error(
            ProviderId::Cursor,
            "fetch failed: cookie=abc123 token=xyz secret=hunter2",
            "2026-06-04T12:00:00Z".to_string(),
        ));
        let content = render_text(&states, None);
        assert!(!content.contains("cookie"));
        assert!(!content.contains("token"));
        assert!(!content.contains("hunter2"));
        assert!(!content.contains("secret"));
    }

    #[test]
    fn dashboard_renders_empty_state() {
        // Table column widths truncate the full sentence; assert the visible prefix.
        let content = render_text(&[], None);
        assert!(content.contains("No usage"));
    }

    #[test]
    fn dashboard_failed_refresh_renders_indicator_and_success_clears_it() {
        let states = fixture_states();
        let mut error: Option<String> = None;
        let kept = apply_refresh(
            states.clone(),
            Err(anyhow::anyhow!("cannot open usage database")),
            &mut error,
        );
        assert_eq!(kept.len(), states.len());
        let shown = error.clone().expect("refresh error recorded");
        let content = render_text(&kept, Some(&shown));
        assert!(content.contains("refresh failed"));
        assert!(content.contains("showing cached data"));

        let refreshed = apply_refresh(states.clone(), Ok(states.clone()), &mut error);
        assert!(error.is_none());
        let content = render_text(&refreshed, error.as_deref());
        assert!(!content.contains("refresh failed"));
    }

    #[test]
    fn dashboard_brief_error_keeps_first_line_only() {
        assert_eq!(
            brief_refresh_error("cannot open usage database\nsecond line"),
            "cannot open usage database"
        );
    }

    #[test]
    fn dashboard_q_esc_and_ctrl_c_quit() {
        assert_eq!(
            handle_dashboard_key(KeyCode::Char('q')),
            DashboardKeyAction::Quit
        );
        assert_eq!(
            handle_dashboard_key(KeyCode::Char('Q')),
            DashboardKeyAction::Quit
        );
        assert_eq!(handle_dashboard_key(KeyCode::Esc), DashboardKeyAction::Quit);
        assert_eq!(
            handle_dashboard_key_event(ctrl_c()),
            DashboardKeyAction::Quit
        );
    }

    #[test]
    fn dashboard_r_refreshes_and_other_keys_are_ignored() {
        assert_eq!(
            handle_dashboard_key(KeyCode::Char('r')),
            DashboardKeyAction::Refresh
        );
        assert_eq!(
            handle_dashboard_key(KeyCode::Char('R')),
            DashboardKeyAction::Refresh
        );
        assert_eq!(
            handle_dashboard_key(KeyCode::Enter),
            DashboardKeyAction::Ignore
        );
    }

    #[test]
    fn dashboard_used_percent_clamps_out_of_range() {
        assert_eq!(format_used_percent(64.4), "64%");
        assert_eq!(format_used_percent(300.0), "100%");
        assert_eq!(format_used_percent(-20.0), "0%");
    }
}
