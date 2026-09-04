//! Config wizard TUI: provider list → cookie-source detail → review.
//! The secret paste buffer renders masked and is never logged; cancelling
//! (Esc/q) leaves settings untouched.

use std::path::Path;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
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

/// One provider's reviewed edits. `manual_cookie` is secret and never
/// appears in `Debug` output.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct WizardEdit {
    pub provider: String,
    pub cookie_source: String,
    pub manual_cookie: String,
}

impl std::fmt::Debug for WizardEdit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let secret = if self.manual_cookie.is_empty() {
            ""
        } else {
            "[redacted]"
        };
        f.debug_struct("WizardEdit")
            .field("provider", &self.provider)
            .field("cookie_source", &self.cookie_source)
            .field("manual_cookie", &secret)
            .finish()
    }
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

    fn current_source(&self) -> &str {
        COOKIE_SOURCES[self.source_index]
    }

    /// Switch the source cursor, dropping any typed secret the moment the
    /// new source stops being `manual`.
    fn set_source_index(&mut self, index: usize) {
        self.source_index = index.min(COOKIE_SOURCES.len() - 1);
        if self.current_source() != "manual" {
            self.secret.clear();
        }
    }

    /// Draft edit for the current detail state. The secret is only carried
    /// when the source is `manual`; any residual buffer under another source
    /// is dropped here and again at confirm time.
    fn detail_edit(&self) -> Option<WizardEdit> {
        let provider = self.current_provider().map(str::to_string)?;
        let manual_cookie = if self.current_source() == "manual" {
            self.secret.clone()
        } else {
            String::new()
        };
        Some(WizardEdit {
            provider,
            cookie_source: self.current_source().to_string(),
            manual_cookie,
        })
    }

    /// Upsert the current detail state into `edits` so back-navigation
    /// preserves in-progress secret/source selections.
    fn save_detail_draft(&mut self) {
        if let Some(edit) = self.detail_edit() {
            match self.edits.iter_mut().find(|e| e.provider == edit.provider) {
                Some(existing) => *existing = edit,
                None => self.edits.push(edit),
            }
        }
    }

    #[cfg(test)]
    pub fn source_for_test(&self) -> &str {
        self.current_source()
    }

    #[cfg(test)]
    pub fn edits_for_test(&mut self, edits: Vec<WizardEdit>) {
        self.edits = edits;
    }

    /// Advance: list → detail (loading any draft), detail → review (saving
    /// the draft), review → done (guarded: empty review stays put).
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
                // Drafts predating the source gate may carry a residual
                // secret under a non-manual source; drop it on load.
                if self.current_source() != "manual" {
                    self.secret.clear();
                }
                self.step = WizardStep::ProviderDetail;
            }
            WizardStep::ProviderDetail => {
                let had_provider = self.current_provider().is_some();
                if had_provider {
                    self.save_detail_draft();
                    self.secret.clear();
                }
                self.step = WizardStep::Review;
            }
            WizardStep::Review => {
                if !self.edits.is_empty() {
                    self.step = WizardStep::Done;
                }
            }
            WizardStep::Done | WizardStep::Cancelled => {}
        }
    }

    /// Step back: detail → list (preserving the in-progress draft),
    /// review → list. From the list, back cancels.
    pub fn back(&mut self) {
        self.step = match self.step {
            WizardStep::ProviderList => WizardStep::Cancelled,
            WizardStep::ProviderDetail => {
                self.save_detail_draft();
                WizardStep::ProviderList
            }
            WizardStep::Review => WizardStep::ProviderList,
            step => step,
        };
    }

    /// Persist reviewed edits through the same setters as `run_config_set`.
    /// Pasted cookies bypass the CLI setter (it refuses secrets by design)
    /// and go through the settings store directly; values are never logged.
    /// Atomic: the settings file is snapshotted first and restored
    /// byte-identical on any mid-loop failure.
    pub fn confirm(&mut self, dir: &Path) -> anyhow::Result<Vec<String>> {
        let path = settings_file_path(dir);
        let snapshot = std::fs::read(&path).ok();
        let result = self.confirm_inner(dir);
        if result.is_err() {
            match snapshot {
                Some(bytes) => {
                    let _ = std::fs::write(&path, bytes);
                }
                None => {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
        result
    }

    fn confirm_inner(&mut self, dir: &Path) -> anyhow::Result<Vec<String>> {
        let mut saved = Vec::new();
        let mut secrets = Vec::new();
        for edit in &self.edits {
            let key = format!("{}.cookie_source", edit.provider);
            saved.push(run_config_set(dir, &key, &edit.cookie_source)?);
            if edit.cookie_source == "manual" && !edit.manual_cookie.trim().is_empty() {
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

    /// Full key-event entry point: Ctrl+C cancels from any live step
    /// (the `KeyCode`-only path cannot see modifiers). Other keys delegate
    /// to [`Self::handle_key`].
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
        {
            match self.step {
                WizardStep::ProviderList | WizardStep::ProviderDetail | WizardStep::Review => {
                    self.step = WizardStep::Cancelled;
                }
                WizardStep::Done | WizardStep::Cancelled => {}
            }
            return;
        }
        self.handle_key(key.code);
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
                KeyCode::Esc => self.back(),
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.step = WizardStep::Cancelled;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.set_source_index(self.source_index.saturating_sub(1));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.set_source_index(self.source_index + 1);
                }
                KeyCode::Enter => self.next(),
                KeyCode::Backspace => {
                    self.secret.pop();
                }
                KeyCode::Char(c) if self.current_source() == "manual" => {
                    self.secret.push(c);
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
        rows.push(if self.current_source() == "manual" {
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
            "↑↓/jk source · type to paste · Enter save · Esc back · q cancel",
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
                        wizard.handle_key_event(key);
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

    use super::{render_config_wizard, ConfigWizard, WizardEdit, WizardStep};

    fn test_dir(name: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "mochi-wizard-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).expect("test dir");
        dir
    }

    fn render_text(wizard: &ConfigWizard) -> String {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_config_wizard(frame, wizard))
            .expect("draw");
        terminal.backend().to_string()
    }

    /// Enter the detail step for the currently selected provider.
    fn enter_detail(wizard: &mut ConfigWizard) {
        assert_eq!(wizard.step, WizardStep::ProviderList);
        wizard.handle_key(KeyCode::Enter);
        assert_eq!(wizard.step, WizardStep::ProviderDetail);
    }

    /// Move the source cursor to `manual` via Down keys.
    fn select_manual(wizard: &mut ConfigWizard) {
        wizard.handle_key(KeyCode::Down);
        assert_eq!(wizard.source_for_test(), "manual");
    }

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

    #[test]
    fn secret_cleared_when_source_switches_away_from_manual() {
        let mut wizard = ConfigWizard::new(vec!["cursor".to_string()]);
        enter_detail(&mut wizard);
        select_manual(&mut wizard);
        for c in "s3cr3t".chars() {
            wizard.handle_key(KeyCode::Char(c));
        }
        assert_eq!(wizard.masked_secret(), "••••••");
        // Switch away: buffer must be cleared.
        wizard.handle_key(KeyCode::Up);
        assert_eq!(wizard.source_for_test(), "browser");
        assert_eq!(wizard.masked_secret(), "");
        // Confirm via detail -> review -> done: no manual_cookie persisted.
        wizard.handle_key(KeyCode::Enter);
        assert_eq!(wizard.step, WizardStep::Review);
        let dir = test_dir("source-switch");
        wizard.handle_key(KeyCode::Enter);
        let saved = wizard.confirm(&dir).expect("confirm");
        assert!(saved
            .iter()
            .any(|line| line.contains("cursor.cookie_source")));
        let settings = crate::settings::load_settings(&crate::settings::settings_file_path(&dir));
        let cookie = settings
            .provider_configs
            .get("cursor")
            .and_then(|config| config.manual_cookie.clone());
        assert_eq!(cookie, None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn debug_output_redacts_manual_cookie() {
        let edit = WizardEdit {
            provider: "cursor".to_string(),
            cookie_source: "manual".to_string(),
            manual_cookie: "super-secret-value".to_string(),
        };
        let rendered = format!("{edit:?}");
        assert!(!rendered.contains("super-secret-value"));
        assert!(rendered.contains("cursor"));
    }

    #[test]
    fn back_preserves_in_progress_detail_edits() {
        let mut wizard = ConfigWizard::new(vec!["cursor".to_string()]);
        enter_detail(&mut wizard);
        select_manual(&mut wizard);
        for c in "abc".chars() {
            wizard.handle_key(KeyCode::Char(c));
        }
        wizard.handle_key(KeyCode::Esc);
        assert_eq!(wizard.step, WizardStep::ProviderList);
        enter_detail(&mut wizard);
        assert_eq!(wizard.source_for_test(), "manual");
        assert_eq!(wizard.masked_secret(), "•••");
    }

    #[test]
    fn detail_q_cancels_without_polluting_secret() {
        let mut wizard = ConfigWizard::new(vec!["cursor".to_string()]);
        enter_detail(&mut wizard);
        select_manual(&mut wizard);
        wizard.handle_key(KeyCode::Char('q'));
        assert_eq!(wizard.step, WizardStep::Cancelled);
        assert!(wizard.confirmed_edits().is_none());
        // Fresh wizard: q from a clean manual detail cancels, inserts nothing.
        let mut wizard = ConfigWizard::new(vec!["cursor".to_string()]);
        enter_detail(&mut wizard);
        select_manual(&mut wizard);
        wizard.handle_key(KeyCode::Char('x'));
        wizard.handle_key(KeyCode::Char('q'));
        assert_eq!(wizard.step, WizardStep::Cancelled);
    }

    #[test]
    fn detail_jk_move_source_cursor() {
        let mut wizard = ConfigWizard::new(vec!["cursor".to_string()]);
        enter_detail(&mut wizard);
        assert_eq!(wizard.source_for_test(), "browser");
        wizard.handle_key(KeyCode::Char('j'));
        assert_eq!(wizard.source_for_test(), "manual");
        wizard.handle_key(KeyCode::Char('k'));
        assert_eq!(wizard.source_for_test(), "browser");
    }

    #[test]
    fn detail_ctrl_c_cancels() {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
        let mut wizard = ConfigWizard::new(vec!["cursor".to_string()]);
        enter_detail(&mut wizard);
        select_manual(&mut wizard);
        wizard.handle_key_event(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        });
        assert_eq!(wizard.step, WizardStep::Cancelled);
    }

    #[test]
    fn rendered_detail_masks_secret() {
        let mut wizard = ConfigWizard::new(vec!["cursor".to_string()]);
        enter_detail(&mut wizard);
        select_manual(&mut wizard);
        for c in "hunter2".chars() {
            wizard.handle_key(KeyCode::Char(c));
        }
        let content = render_text(&wizard);
        assert!(!content.contains("hunter2"));
        assert!(content.contains("•••••••"));
    }

    #[test]
    fn multibyte_secret_masks_per_char() {
        let mut wizard = ConfigWizard::new(vec!["cursor".to_string()]);
        enter_detail(&mut wizard);
        select_manual(&mut wizard);
        for c in "héllo😀".chars() {
            wizard.handle_key(KeyCode::Char(c));
        }
        assert_eq!(wizard.masked_secret(), "••••••");
        let content = render_text(&wizard);
        assert!(!content.contains("héllo"));
    }

    #[test]
    fn review_masks_secret() {
        let mut wizard = ConfigWizard::new(vec!["cursor".to_string()]);
        enter_detail(&mut wizard);
        select_manual(&mut wizard);
        for c in "hunter2".chars() {
            wizard.handle_key(KeyCode::Char(c));
        }
        wizard.handle_key(KeyCode::Enter);
        assert_eq!(wizard.step, WizardStep::Review);
        let content = render_text(&wizard);
        assert!(!content.contains("hunter2"));
    }

    #[test]
    fn cancel_with_edits_leaves_settings_untouched() {
        let dir = test_dir("cancel-untouched");
        let path = crate::settings::settings_file_path(&dir);
        crate::cli::config::run_config_set(&dir, "update_channel", "stable").expect("seed");
        let before = std::fs::read(&path).expect("read seed");
        let mut wizard = ConfigWizard::new(vec!["cursor".to_string()]);
        enter_detail(&mut wizard);
        select_manual(&mut wizard);
        for c in "hunter2".chars() {
            wizard.handle_key(KeyCode::Char(c));
        }
        wizard.handle_key(KeyCode::Enter);
        assert_eq!(wizard.step, WizardStep::Review);
        wizard.handle_key(KeyCode::Char('q'));
        assert_eq!(wizard.step, WizardStep::Cancelled);
        assert!(wizard.confirmed_edits().is_none());
        assert_eq!(std::fs::read(&path).expect("read after"), before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn confirm_persists_reviewed_values() {
        let dir = test_dir("confirm-persists");
        let mut wizard = ConfigWizard::new(vec!["cursor".to_string()]);
        enter_detail(&mut wizard);
        select_manual(&mut wizard);
        for c in "hunter2".chars() {
            wizard.handle_key(KeyCode::Char(c));
        }
        wizard.handle_key(KeyCode::Enter);
        assert_eq!(wizard.step, WizardStep::Review);
        wizard.handle_key(KeyCode::Enter);
        let saved = wizard.confirm(&dir).expect("confirm");
        assert!(saved
            .iter()
            .any(|line| line.contains("cursor.cookie_source")));
        let settings = crate::settings::load_settings(&crate::settings::settings_file_path(&dir));
        let config = settings
            .provider_configs
            .get("cursor")
            .expect("cursor config");
        assert_eq!(config.cookie_source.as_deref(), Some("manual"));
        assert_eq!(config.manual_cookie.as_deref(), Some("hunter2"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn confirm_restores_snapshot_on_mid_loop_failure() {
        let dir = test_dir("confirm-atomic");
        crate::cli::config::run_config_set(&dir, "cursor.cookie_source", "browser").expect("seed");
        let path = crate::settings::settings_file_path(&dir);
        let before = std::fs::read(&path).expect("read seed");
        let mut wizard = ConfigWizard::new(vec!["cursor".to_string()]);
        wizard.edits_for_test(vec![
            WizardEdit {
                provider: "cursor".to_string(),
                cookie_source: "manual".to_string(),
                manual_cookie: String::new(),
            },
            WizardEdit {
                provider: "nope-provider".to_string(),
                cookie_source: "manual".to_string(),
                manual_cookie: String::new(),
            },
        ]);
        let err = wizard.confirm(&dir).expect_err("second edit fails");
        assert!(err.to_string().contains("unknown key"));
        assert_eq!(std::fs::read(&path).expect("read after"), before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_review_enter_stays_on_review() {
        let mut wizard = ConfigWizard::new(vec![]);
        wizard.handle_key(KeyCode::Enter);
        assert_eq!(wizard.step, WizardStep::Review);
        wizard.handle_key(KeyCode::Enter);
        assert_eq!(wizard.step, WizardStep::Review);
        assert!(wizard.confirmed_edits().is_none());
    }
}
