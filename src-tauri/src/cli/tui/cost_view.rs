//! Cost view TUI: read-only spend lines with per-currency formatting.
//!
//! Money lines only — `CostEntry` carries no credentials and nothing here
//! mutates state.

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, Paragraph},
    Frame, Terminal,
};

use super::{install_panic_hook, usage_dashboard::cost_period_label, TuiGuard};
use crate::cli::cost::{format_cost_detail, CostEntry};
use crate::tray::provider_display_name;

/// Pure render over `&[CostEntry]`; TestBackend-compatible (no terminal I/O).
pub fn render_cost_view(frame: &mut Frame, entries: &[CostEntry]) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
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
    frame.render_widget(Paragraph::new("q/Esc quit · r refresh"), chunks[1]);
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
    // Install LAST so terminal restore runs before the diagnostics logger.
    install_panic_hook();
    {
        let _guard = TuiGuard::enter()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        loop {
            terminal.draw(|frame| render_cost_view(frame, &entries))?;
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
                                entries = load(filter).unwrap_or(entries);
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

    use super::render_cost_view;
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

    #[test]
    fn cost_view_shows_money_line() {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_cost_view(frame, &fixture_costs()))
            .expect("draw");
        assert!(terminal.backend().to_string().contains("$7.54 / $71.93"));
    }

    #[test]
    fn cost_view_uses_widget_period_label() {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_cost_view(frame, &fixture_costs()))
            .expect("draw");
        let content = terminal.backend().to_string();
        assert!(content.contains("Billing period"));
        assert!(!content.contains("billing-period"));
    }
}
