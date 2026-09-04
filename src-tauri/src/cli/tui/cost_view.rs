//! Cost view TUI: read-only spend lines with per-currency formatting.
//!
//! Money lines only — `CostEntry` carries no credentials and nothing here
//! mutates state.

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, Paragraph},
    Frame, Terminal,
};

use super::{install_panic_hook, usage_dashboard::apply_refresh, TuiGuard};
use crate::cli::cost::{cost_period_label, format_cost_detail, CostEntry};
use crate::tray::provider_display_name;

/// Unit-testable key decision for the cost loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostKeyAction {
    Quit,
    Refresh,
    Ignore,
}

/// Full key-event entry point: Ctrl+C quits from anywhere
/// (the `KeyCode`-only path cannot see modifiers).
pub fn handle_cost_key_event(key: KeyEvent) -> CostKeyAction {
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
    {
        return CostKeyAction::Quit;
    }
    handle_cost_key(key.code)
}

pub fn handle_cost_key(code: KeyCode) -> CostKeyAction {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => CostKeyAction::Quit,
        KeyCode::Char('r') | KeyCode::Char('R') => CostKeyAction::Refresh,
        _ => CostKeyAction::Ignore,
    }
}

/// Thin typed fold over the shared [`apply_refresh`]; failure keeps the
/// stale entries and records the brief cause for the on-screen indicator.
pub fn apply_cost_refresh(
    current: Vec<CostEntry>,
    result: anyhow::Result<Vec<CostEntry>>,
    refresh_error: &mut Option<String>,
) -> Vec<CostEntry> {
    apply_refresh(current, result, refresh_error)
}

/// Pure render over `&[CostEntry]`; TestBackend-compatible (no terminal I/O).
/// `refresh_error` (when `Some`) renders a visible failure indicator while the
/// stale list stays on screen.
pub fn render_cost_view(frame: &mut Frame, entries: &[CostEntry], refresh_error: Option<&str>) {
    let area = frame.area();
    let failed = refresh_error.map(str::trim).filter(|s| !s.is_empty());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(match failed {
            Some(_) => [Constraint::Min(1), Constraint::Length(2)],
            None => [Constraint::Min(1), Constraint::Length(1)],
        })
        .split(area);
    let rows: Vec<String> = if entries.is_empty() {
        vec!["No cost data.".to_string()]
    } else {
        entries
            .iter()
            .map(|entry| {
                format!(
                    "{} {} ({})",
                    provider_display_name(entry.provider),
                    format_cost_detail(entry.used, entry.limit, &entry.currency_code),
                    cost_period_label(Some(&entry.period)),
                )
            })
            .collect()
    };
    let list = List::new(rows).block(Block::default().title("Mochi cost").borders(Borders::ALL));
    frame.render_widget(list, chunks[0]);
    let footer = match failed {
        Some(cause) => {
            format!("q/Esc quit · r refresh\n(refresh failed: {cause} — showing cached data)")
        }
        None => "q/Esc quit · r refresh".to_string(),
    };
    frame.render_widget(Paragraph::new(footer), chunks[1]);
}

/// Run the fullscreen cost event loop. Read-only: q/Esc exits and `r`
/// re-reads the cached store (no live fetch, no mutations).
pub fn run_cost_view(provider: Option<&str>, days: u16) -> anyhow::Result<()> {
    let filter = provider
        .map(crate::cli::cost::parse_provider_filter)
        .transpose()?;
    let load = |filter: Option<crate::core::models::ProviderId>| -> anyhow::Result<Vec<CostEntry>> {
        let entries = crate::cli::cost::load_cost_entries(days)?;
        Ok(match filter {
            Some(id) => entries.into_iter().filter(|e| e.provider == id).collect(),
            None => entries,
        })
    };
    let mut entries = load(filter)?;
    let mut refresh_error: Option<String> = None;
    // Install LAST so terminal restore runs before the diagnostics logger.
    install_panic_hook();
    {
        let _guard = TuiGuard::enter()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        loop {
            terminal.draw(|frame| render_cost_view(frame, &entries, refresh_error.as_deref()))?;
            if event::poll(Duration::from_millis(250))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        match handle_cost_key_event(key) {
                            CostKeyAction::Quit => break,
                            CostKeyAction::Refresh => {
                                entries =
                                    apply_cost_refresh(entries, load(filter), &mut refresh_error);
                            }
                            CostKeyAction::Ignore => {}
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
        apply_cost_refresh, handle_cost_key, handle_cost_key_event, render_cost_view, CostKeyAction,
    };
    use crate::cli::cost::CostEntry;
    use crate::core::models::ProviderId;

    fn fixture_costs() -> Vec<CostEntry> {
        vec![CostEntry {
            provider: ProviderId::CommandCode,
            used: 7.54,
            limit: 71.93,
            currency_code: "USD".to_string(),
            period: "billing-period".to_string(),
        }]
    }

    fn render_text(entries: &[CostEntry], refresh_error: Option<&str>) -> String {
        // 80 wide: fits the two-line refresh-failed footer without truncation.
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_cost_view(frame, entries, refresh_error))
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
    fn cost_view_shows_money_line() {
        assert!(render_text(&fixture_costs(), None).contains("$7.54 / $71.93"));
    }

    #[test]
    fn cost_view_uses_widget_period_label() {
        let content = render_text(&fixture_costs(), None);
        assert!(content.contains("Billing period"));
        assert!(!content.contains("billing-period"));
    }

    #[test]
    fn cost_view_renders_empty_state() {
        assert!(render_text(&[], None).contains("No cost data."));
    }

    #[test]
    fn cost_view_failed_refresh_renders_indicator_and_success_clears_it() {
        let entries = fixture_costs();
        let mut error: Option<String> = None;
        let kept = apply_cost_refresh(
            entries.clone(),
            Err(anyhow::anyhow!("cannot open usage database")),
            &mut error,
        );
        assert_eq!(kept.len(), entries.len());
        let shown = error.clone().expect("refresh error recorded");
        let content = render_text(&kept, Some(&shown));
        assert!(content.contains("refresh failed"));
        assert!(content.contains("showing cached data"));

        let refreshed = apply_cost_refresh(entries.clone(), Ok(entries.clone()), &mut error);
        assert!(error.is_none());
        let content = render_text(&refreshed, error.as_deref());
        assert!(!content.contains("refresh failed"));
    }

    #[test]
    fn cost_view_q_esc_and_ctrl_c_quit() {
        assert_eq!(handle_cost_key(KeyCode::Char('q')), CostKeyAction::Quit);
        assert_eq!(handle_cost_key(KeyCode::Char('Q')), CostKeyAction::Quit);
        assert_eq!(handle_cost_key(KeyCode::Esc), CostKeyAction::Quit);
        assert_eq!(handle_cost_key_event(ctrl_c()), CostKeyAction::Quit);
    }

    #[test]
    fn cost_view_r_refreshes_and_other_keys_are_ignored() {
        assert_eq!(handle_cost_key(KeyCode::Char('r')), CostKeyAction::Refresh);
        assert_eq!(handle_cost_key(KeyCode::Char('R')), CostKeyAction::Refresh);
        assert_eq!(handle_cost_key(KeyCode::Enter), CostKeyAction::Ignore);
    }
}
