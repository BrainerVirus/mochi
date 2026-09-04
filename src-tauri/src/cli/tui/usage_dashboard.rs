//! Usage dashboard TUI: read-only provider usage table.
//!
//! Percentages and window labels only — snapshot messages (which can carry
//! fetch errors) and secrets are never rendered.

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame, Terminal,
};

use super::{install_panic_hook, TuiGuard};
use crate::core::usage_state::ProviderUsageState;
use crate::tray::provider_display_name;

/// Mirror of the frontend `formatCostPeriodLabel`: raw period ids
/// ("billing-period") never render as labels; human labels
/// ("Billing period") do, with "On-demand" for missing periods.
pub(crate) fn cost_period_label(period: Option<&str>) -> String {
    let raw = period.map(str::trim).unwrap_or("");
    let words: Vec<&str> = raw.split('-').filter(|word| !word.is_empty()).collect();
    let [first, rest @ ..] = words.as_slice() else {
        return "On-demand".to_string();
    };
    let mut label = String::with_capacity(raw.len() + 1);
    let mut chars = first.chars();
    match chars.next() {
        Some(head) => label.extend(head.to_uppercase()),
        None => return "On-demand".to_string(),
    }
    label.push_str(chars.as_str());
    for word in rest {
        label.push(' ');
        label.push_str(word);
    }
    label
}

/// Pure render over `&[ProviderUsageState]`; TestBackend-compatible (no terminal I/O).
pub fn render_usage_dashboard(frame: &mut Frame, states: &[ProviderUsageState]) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
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
                Cell::from(format!("{}%", window.used_percent.round() as u8)),
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
    frame.render_widget(Paragraph::new("q/Esc quit · r refresh"), chunks[1]);
}

/// Run the fullscreen dashboard event loop. Read-only: q/Esc exits and `r`
/// re-reads the cached store (no live fetch, no mutations).
pub fn run_usage_dashboard(provider: Option<&str>, refresh: bool) -> anyhow::Result<()> {
    let mut states = crate::cli_usage_states(provider, refresh)?;
    // Install LAST so terminal restore runs before the diagnostics logger.
    install_panic_hook();
    {
        let _guard = TuiGuard::enter()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        loop {
            terminal.draw(|frame| render_usage_dashboard(frame, &states))?;
            if event::poll(Duration::from_millis(250))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
                        {
                            break;
                        }
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                states = crate::cli_usage_states(provider, false).unwrap_or(states);
                            }
                            _ => {}
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
    use ratatui::{backend::TestBackend, Terminal};

    use super::{cost_period_label, render_usage_dashboard};
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

    #[test]
    fn dashboard_uses_widget_labels() {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_usage_dashboard(frame, &fixture_states()))
            .expect("draw");
        let content = terminal.backend().to_string();
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
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_usage_dashboard(frame, &states))
            .expect("draw");
        let content = terminal.backend().to_string();
        assert!(!content.contains("cookie"));
        assert!(!content.contains("token"));
        assert!(!content.contains("hunter2"));
        assert!(!content.contains("secret"));
    }

    #[test]
    fn cost_period_label_formats_like_widget() {
        assert_eq!(
            cost_period_label(Some("billing-period")),
            "Billing period".to_string()
        );
        assert_eq!(cost_period_label(None), "On-demand".to_string());
        assert_eq!(cost_period_label(Some("")), "On-demand".to_string());
        assert_eq!(cost_period_label(Some("--")), "On-demand".to_string());
    }
}
