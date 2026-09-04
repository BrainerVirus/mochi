//! Config wizard TUI: provider list → cookie-source detail → review.
//! The secret paste buffer renders masked and is never logged; cancelling
//! (Esc/q) leaves settings untouched.

use std::path::Path;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, Paragraph},
    Frame, Terminal,
};

use super::{install_panic_hook, screens::ListNav, TuiGuard};
use crate::cli::config::run_config_set;
use crate::core::models::ProviderId;
use crate::settings::{load_settings, persist_settings, settings_file_path};

/// Cookie-source radio options shown in the detail step.
const COOKIE_SOURCES: [&str; 3] = ["browser", "manual", "off"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WizardStep {
    #[default]
    ProviderList,
    ProviderDetail,
    Review,
    Done,
    Cancelled,
}

/// One provider's reviewed edits. `manual_cookie` is secret.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WizardEdit {
    pub provider: String,
    pub cookie_source: String,
    pub manual_cookie: String,
}

pub struct ConfigWizard {
    pub step: WizardStep,
    pub providers: Vec<String>,
    pub edits: Vec<WizardEdit>,
    selected: ListNav,
    source_index: usize,
    secret: String,
}

impl ConfigWizard {
    pub fn new(providers: Vec<String>) -> Self {
        Self {
            step: WizardStep::ProviderList,
            selected: ListNav::new(providers.len()),
            providers,
            edits: Vec::new(),
            source_index: 0,
            secret: String::new(),
        }
    }

    fn current_provider(&self) -> Option<&str> {
        self.providers
            .get(self.selected.selected())
            .map(String::as_str)
    }

    /// Advance: list → detail (loading any draft), detail → review (saving
    /// the draft), review → done.
    pub fn next(&mut self) {
        match self.step {
            WizardStep::ProviderList => {
                let Some(current) = self.current_provider().map(str::to_string) else {
                    self.step = WizardStep::Review;
                    return;
                };
                let draft = self.edits.iter().find(|edit| edit.provider == current);
                (self.source_index, self.secret) = draft
                    .map(|edit| {
                        (
                            COOKIE_SOURCES
                                .iter()
                                .position(|source| *source == edit.cookie_source)
                                .unwrap_or(0),
                            edit.manual_cookie.clone(),
                        )
                    })
                    .unwrap_or_default();
                self.step = WizardStep::ProviderDetail;
            }
            WizardStep::ProviderDetail => {
                if let Some(provider) = self.current_provider().map(str::to_string) {
                    let edit = WizardEdit {
                        provider,
                        cookie_source: COOKIE_SOURCES[self.source_index].to_string(),
                        manual_cookie: std::mem::take(&mut self.secret),
                    };
                    match self.edits.iter_mut().find(|e| e.provider == edit.provider) {
                        Some(existing) => *existing = edit,
                        None => self.edits.push(edit),
                    }
                }
                self.step = WizardStep::Review;
            }
            WizardStep::Review => self.step = WizardStep::Done,
            WizardStep::Done | WizardStep::Cancelled => {}
        }
    }

    /// Step back: detail/review → list. From the list, back cancels.
    pub fn back(&mut self) {
        self.step = match self.step {
            WizardStep::ProviderList => WizardStep::Cancelled,
            WizardStep::ProviderDetail | WizardStep::Review => WizardStep::ProviderList,
            step => step,
        };
    }

    /// Persist reviewed edits through the same setters as `run_config_set`.
    /// Pasted cookies bypass the CLI setter (it refuses secrets by design)
    /// and go through the settings store directly; values are never logged.
    pub fn confirm(&mut self, dir: &Path) -> anyhow::Result<Vec<String>> {
        let mut saved = Vec::new();
        let mut secrets = Vec::new();
        for edit in &self.edits {
            let key = format!("{}.cookie_source", edit.provider);
            saved.push(run_config_set(dir, &key, &edit.cookie_source)?);
            if !edit.manual_cookie.trim().is_empty() {
                secrets.push((edit.provider.clone(), edit.manual_cookie.clone()));
            }
        }
        if !secrets.is_empty() {
            let path = settings_file_path(dir);
            let mut settings = load_settings(&path);
            for (provider, cookie) in &secrets {
                settings
                    .provider_configs
                    .entry(provider.clone())
                    .or_default()
                    .manual_cookie = Some(cookie.clone());
            }
            settings.normalize_provider_ids();
            persist_settings(&path, &settings).map_err(|error| anyhow::anyhow!(error))?;
            saved.push(format!("{} provider secret(s) stored", secrets.len()));
        }
        self.step = WizardStep::Done;
        Ok(saved)
    }

    /// Reviewed edits once finished; `None` while running or when cancelled
    /// (Esc/q) so callers never persist on abort.
    pub fn confirmed_edits(&self) -> Option<&[WizardEdit]> {
        (self.step == WizardStep::Done).then_some(self.edits.as_slice())
    }

    /// Masked rendering of the secret buffer. Length only, never contents.
    pub fn masked_secret(&self) -> String {
        "•".repeat(self.secret.chars().count())
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.step {
            WizardStep::ProviderList => match key {
                KeyCode::Up | KeyCode::Char('k') => self.selected.move_up(),
                KeyCode::Down | KeyCode::Char('j') => self.selected.move_down(),
                KeyCode::Enter => self.next(),
                KeyCode::Esc | KeyCode::Char('q') => self.back(),
                _ => {}
            },
            WizardStep::ProviderDetail => match key {
                KeyCode::Up => self.source_index = self.source_index.saturating_sub(1),
                KeyCode::Down => {
                    self.source_index = (self.source_index + 1).min(COOKIE_SOURCES.len() - 1);
                }
                KeyCode::Enter => self.next(),
                KeyCode::Esc => self.back(),
                KeyCode::Char(c) if COOKIE_SOURCES[self.source_index] == "manual" => {
                    self.secret.push(c);
                }
                KeyCode::Backspace => {
                    self.secret.pop();
                }
                _ => {}
            },
            WizardStep::Review => match key {
                KeyCode::Enter => self.next(),
                KeyCode::Esc | KeyCode::Char('b') => self.back(),
                KeyCode::Char('q') => self.step = WizardStep::Cancelled,
                _ => {}
            },
            WizardStep::Done | WizardStep::Cancelled => {}
        }
    }

    fn list_rows(&self) -> Vec<String> {
        let mut rows = Vec::with_capacity(self.providers.len());
        if self.providers.is_empty() {
            rows.push("(no providers enabled)".to_string());
        }
        for (i, provider) in self.providers.iter().enumerate() {
            let mark = if i == self.selected.selected() {
                "> "
            } else {
                "  "
            };
            rows.push(format!("{mark}{provider}"));
        }
        rows
    }

    fn detail_rows(&self) -> Vec<String> {
        let mut rows = Vec::with_capacity(COOKIE_SOURCES.len() + 2);
        for (i, source) in COOKIE_SOURCES.iter().enumerate() {
            let mark = if i == self.source_index {
                "(*) "
            } else {
                "( ) "
            };
            rows.push(format!("{mark}{source}"));
        }
        rows.push(String::new());
        rows.push(if COOKIE_SOURCES[self.source_index] == "manual" {
            format!("cookie: {}", self.masked_secret())
        } else {
            "select manual to paste a cookie".to_string()
        });
        rows
    }

    fn review_rows(&self) -> Vec<String> {
        if self.edits.is_empty() {
            return vec!["No changes yet.".to_string()];
        }
        self.edits
            .iter()
            .map(|e| {
                let secret = if e.manual_cookie.trim().is_empty() {
                    ""
                } else {
                    " + cookie"
                };
                format!("{}: cookie_source={}{secret}", e.provider, e.cookie_source)
            })
            .collect()
    }
}

/// Pure render over `&ConfigWizard`; TestBackend-compatible (no terminal I/O).
pub fn render_config_wizard(frame: &mut Frame, wizard: &ConfigWizard) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);
    let current = wizard.current_provider().unwrap_or("(none)").to_string();
    let (title, rows, help) = match wizard.step {
        WizardStep::ProviderList => (
            format!("Mochi config — providers ({})", wizard.providers.len()),
            wizard.list_rows(),
            "↑↓/jk move · Enter select · q quit",
        ),
        WizardStep::ProviderDetail => (
            format!("Mochi config — {current}"),
            wizard.detail_rows(),
            "↑↓ source · type to paste · Enter save · Esc back",
        ),
        WizardStep::Review => (
            "Mochi config — review".to_string(),
            wizard.review_rows(),
            "Enter confirm · b back · q cancel",
        ),
        WizardStep::Done | WizardStep::Cancelled => (
            "Mochi config".to_string(),
            vec![if wizard.step == WizardStep::Done {
                "Configuration saved.".to_string()
            } else {
                "Cancelled — settings unchanged.".to_string()
            }],
            "",
        ),
    };
    let list = List::new(rows).block(Block::default().title(title.as_str()).borders(Borders::ALL));
    frame.render_widget(list, chunks[0]);
    frame.render_widget(Paragraph::new(help), chunks[1]);
}

/// Run the fullscreen wizard event loop. Returns after the terminal is
/// restored; prints saved values (never secrets) only on confirm.
pub fn run_config_wizard() -> anyhow::Result<()> {
    // Install LAST so terminal restore runs before the diagnostics logger.
    install_panic_hook();
    let dir =
        crate::cli_config_dir().ok_or_else(|| anyhow::anyhow!("cannot locate config directory"))?;
    let settings = load_settings(&settings_file_path(&dir));
    let mut providers = settings.enabled_providers.clone();
    if providers.is_empty() {
        providers = ProviderId::all()
            .iter()
            .map(|p| p.as_str().to_string())
            .collect();
    }
    let mut wizard = ConfigWizard::new(providers);
    {
        let _guard = TuiGuard::enter()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        loop {
            terminal.draw(|frame| render_config_wizard(frame, &wizard))?;
            if matches!(wizard.step, WizardStep::Done | WizardStep::Cancelled) {
                break;
            }
            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        wizard.handle_key(key.code);
                    }
                }
            }
        }
    }
    if wizard.step == WizardStep::Done {
        for line in wizard.confirm(&dir)? {
            println!("{line}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;
    use ratatui::{backend::TestBackend, Terminal};

    use super::{render_config_wizard, ConfigWizard};

    #[test]
    fn wizard_renders_provider_list() {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let wizard = ConfigWizard::new(vec!["claude".to_string(), "zai".to_string()]);
        terminal
            .draw(|frame| render_config_wizard(frame, &wizard))
            .expect("draw");
        let content = terminal.backend().to_string();
        assert!(content.contains("claude"));
        assert!(content.contains("zai"));
    }

    #[test]
    fn wizard_abort_returns_no_edits() {
        let mut wizard = ConfigWizard::new(vec![]);
        wizard.handle_key(KeyCode::Esc);
        assert!(wizard.confirmed_edits().is_none());
    }
}
