//! Update flow TUI: notes preview → confirm → applied/aborted.
//! Honest semantics (Task 4): the headless CLI never replaces the binary,
//! so Applied reports the version, the installer URL, and the GUI-install
//! path — never "updated to". Esc/q cancels with no mutation (CA-03).

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, Paragraph},
    Frame, Terminal,
};

use super::{install_panic_hook, TuiGuard, EXIT_DOMAIN};
use crate::cli::update::format_apply_output;
use crate::updater::{check_stable_update, UpdateInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlowState {
    #[default]
    Notes,
    Confirm,
    Applied,
    Aborted,
}

pub struct UpdateFlow {
    pub state: FlowState,
    version: Option<String>,
    notes: String,
    download_url: Option<String>,
    output: Option<String>,
    confirmed: bool,
}

impl UpdateFlow {
    pub fn new() -> Self {
        Self {
            state: FlowState::Notes,
            version: None,
            notes: String::new(),
            download_url: None,
            output: None,
            confirmed: false,
        }
    }

    pub fn notes_available(version: &str, notes: &str) -> Self {
        Self {
            state: FlowState::Notes,
            version: Some(version.to_string()),
            notes: notes.to_string(),
            download_url: None,
            output: None,
            confirmed: false,
        }
    }

    pub fn from_info(info: &UpdateInfo) -> Self {
        match (&info.available, &info.version) {
            (true, Some(version)) => Self {
                state: FlowState::Notes,
                version: Some(version.clone()),
                notes: info.notes.clone().unwrap_or_default(),
                download_url: info.download_url.clone(),
                output: None,
                confirmed: false,
            },
            _ => Self {
                state: FlowState::Applied,
                version: None,
                notes: String::new(),
                download_url: None,
                output: Some("up to date".to_string()),
                confirmed: false,
            },
        }
    }

    fn info(&self) -> UpdateInfo {
        UpdateInfo {
            available: self.version.is_some(),
            version: self.version.clone(),
            channel: "stable".to_string(),
            notes: Some(self.notes.clone()).filter(|notes| !notes.trim().is_empty()),
            download_url: self.download_url.clone(),
        }
    }

    /// True only after explicit Enter on the confirm step. Esc paths and the
    /// up-to-date short-circuit never set it, so callers never mutate on abort.
    pub fn applied(&self) -> bool {
        self.confirmed
    }

    pub fn output(&self) -> Option<&str> {
        self.output.as_deref()
    }

    fn abort(&mut self) {
        if matches!(self.state, FlowState::Notes | FlowState::Confirm) {
            self.state = FlowState::Aborted;
        }
    }

    /// Full key-event entry point: Ctrl+C aborts from any live step
    /// (the `KeyCode`-only path cannot see modifiers).
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
        {
            self.abort();
            return;
        }
        self.handle_key(key.code);
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.state {
            FlowState::Notes => match key {
                KeyCode::Enter => self.state = FlowState::Confirm,
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => self.abort(),
                _ => {}
            },
            // Confirm ONLY on explicit Enter; anything else leaves the flow
            // put (Esc/q aborts with no mutation).
            FlowState::Confirm => match key {
                KeyCode::Enter => {
                    self.output = Some(format_apply_output(&self.info()));
                    self.confirmed = true;
                    self.state = FlowState::Applied;
                }
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => self.abort(),
                _ => {}
            },
            FlowState::Applied | FlowState::Aborted => {}
        }
    }

    /// Confirm-step rows: version line, installer URL only when present, and
    /// the GUI-install path. No blank filler, no inline prompt — the footer
    /// owns the single `Enter confirm` prompt.
    fn confirm_rows(&self) -> Vec<String> {
        let version = self.version.as_deref().unwrap_or("(none)");
        let mut rows = vec![format!("{version} available")];
        if let Some(url) = self
            .download_url
            .as_deref()
            .filter(|url| !url.trim().is_empty())
        {
            rows.push(url.to_string());
        }
        rows.push("install via the GUI updater (install_update)".to_string());
        rows
    }
}

impl Default for UpdateFlow {
    fn default() -> Self {
        Self::new()
    }
}

/// Pure render over `&UpdateFlow`; TestBackend-compatible (no terminal I/O).
pub fn render_update_flow(frame: &mut Frame, flow: &UpdateFlow) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);
    let version = flow.version.as_deref().unwrap_or("(none)");
    let (title, rows, help) = match flow.state {
        FlowState::Notes => {
            let mut rows: Vec<String> = flow.notes.lines().map(str::to_string).collect();
            if rows.is_empty() {
                rows.push("(no release notes)".to_string());
            }
            (
                format!("Mochi update — {version} available"),
                rows,
                "Enter confirm · Esc/q cancel",
            )
        }
        FlowState::Confirm => (
            format!("Mochi update — confirm {version}"),
            flow.confirm_rows(),
            "Enter confirm · Esc/q cancel (no change until Enter)",
        ),
        FlowState::Applied => (
            "Mochi update".to_string(),
            flow.output
                .as_deref()
                .unwrap_or("up to date")
                .lines()
                .map(str::to_string)
                .collect(),
            "",
        ),
        FlowState::Aborted => (
            "Mochi update".to_string(),
            vec!["Cancelled — nothing applied.".to_string()],
            "",
        ),
    };
    let list = List::new(rows).block(Block::default().title(title.as_str()).borders(Borders::ALL));
    frame.render_widget(list, chunks[0]);
    frame.render_widget(Paragraph::new(help), chunks[1]);
}

/// Persistent output for a finished flow, formatted from the already-fetched
/// snapshot — never a second live fetch, so feed flap between confirm and
/// apply can neither change nor fail the printed result. `Some` exactly when
/// the flow reached `Applied`, which covers both the confirmed apply and the
/// up-to-date short-circuit. The caller prints it after the alternate-screen
/// restore so it survives on real stdout.
pub fn final_output_for_flow(info: &UpdateInfo, flow: &UpdateFlow) -> Option<String> {
    (flow.state == FlowState::Applied).then(|| format_apply_output(info))
}

/// Map a pre-TUI fetch failure onto the headless arm's shape: the raw feed
/// message plus the domain exit code, without the anyhow `mochi failed:`
/// prefix the `?` early-return would add via `main`.
pub(crate) fn precheck_failure(message: String) -> (String, i32) {
    (message, EXIT_DOMAIN)
}

/// Run the fullscreen update event loop. Returns after the terminal is
/// restored; the printed apply output is formatted from the pre-TUI fetch
/// snapshot through the same [`format_apply_output`] helper as the plain
/// `update apply --confirm` command — no second network fetch.
pub fn run_update_flow() -> anyhow::Result<()> {
    let info = match check_stable_update() {
        Ok(info) => info,
        Err(message) => {
            let (message, code) = precheck_failure(message);
            eprintln!("{message}");
            std::process::exit(code);
        }
    };
    let mut flow = UpdateFlow::from_info(&info);
    // Install LAST and only once the pre-TUI fetch succeeded, just before
    // entering the alternate screen: a pre-TUI panic must never emit
    // leave-screen escapes to a never-entered terminal, and hook order is
    // LIFO so terminal restore still runs before the diagnostics logger.
    install_panic_hook();
    {
        let _guard = TuiGuard::enter()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        loop {
            terminal.draw(|frame| render_update_flow(frame, &flow))?;
            if matches!(flow.state, FlowState::Applied | FlowState::Aborted) {
                break;
            }
            if event::poll(Duration::from_millis(250))? {
                // Read-only flow: no paste input, key events only.
                let evt = event::read()?;
                let Event::Key(key) = evt else { continue };
                if key.kind == KeyEventKind::Press {
                    flow.handle_key_event(key);
                }
            }
        }
    }
    if let Some(output) = final_output_for_flow(&info, &flow) {
        println!("{output}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;
    use ratatui::{backend::TestBackend, Terminal};

    use super::{render_update_flow, FlowState, UpdateFlow};

    fn render_text(flow: &UpdateFlow) -> String {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_update_flow(frame, flow))
            .expect("draw");
        terminal.backend().to_string()
    }

    #[test]
    fn update_flow_renders_notes_preview() {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let flow = UpdateFlow::notes_available("0.3.1", "- fixed tray races");
        terminal
            .draw(|frame| render_update_flow(frame, &flow))
            .expect("draw");
        let content = terminal.backend().to_string();
        assert!(content.contains("0.3.1"));
        assert!(content.contains("confirm"));
    }

    #[test]
    fn update_flow_cancel_applies_nothing() {
        let mut flow = UpdateFlow::notes_available("0.3.1", "notes");
        flow.handle_key(KeyCode::Esc);
        assert!(!flow.applied());
    }

    #[test]
    fn update_flow_confirms_only_on_enter() {
        let mut flow = UpdateFlow::notes_available("0.3.1", "notes");
        flow.handle_key(KeyCode::Enter);
        assert_eq!(flow.state, FlowState::Confirm);
        assert!(!flow.applied());
        flow.handle_key(KeyCode::Char('y'));
        assert!(!flow.applied());
        flow.handle_key(KeyCode::Enter);
        assert!(flow.applied());
    }

    #[test]
    fn update_flow_applied_output_never_claims_install() {
        let mut flow = UpdateFlow::notes_available("0.3.1", "notes");
        flow.handle_key(KeyCode::Enter);
        flow.handle_key(KeyCode::Enter);
        let output = flow.output().expect("applied output");
        assert!(output.contains("0.3.1"), "names the version, got: {output}");
        assert!(
            !output.contains("updated to"),
            "must never claim the binary was replaced, got: {output}"
        );
        assert!(render_text(&flow).contains("0.3.1"));
    }

    #[test]
    fn update_flow_confirm_esc_aborts_without_apply() {
        let mut flow = UpdateFlow::notes_available("0.3.1", "notes");
        flow.handle_key(KeyCode::Enter);
        flow.handle_key(KeyCode::Esc);
        assert_eq!(flow.state, FlowState::Aborted);
        assert!(!flow.applied());
    }

    fn ctrl_c_event() -> crossterm::event::KeyEvent {
        crossterm::event::KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: crossterm::event::KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    fn sample_available_info() -> crate::updater::UpdateInfo {
        crate::updater::UpdateInfo {
            available: true,
            version: Some("0.3.1".to_string()),
            channel: "stable".to_string(),
            notes: Some("- fixed tray races".to_string()),
            download_url: Some("https://example.com/mochi-0.3.1.dmg".to_string()),
        }
    }

    fn up_to_date_info() -> crate::updater::UpdateInfo {
        crate::updater::UpdateInfo {
            available: false,
            version: None,
            channel: "stable".to_string(),
            notes: None,
            download_url: None,
        }
    }

    #[test]
    fn update_flow_q_cancels_from_notes() {
        let mut flow = UpdateFlow::notes_available("0.3.1", "notes");
        flow.handle_key(KeyCode::Char('q'));
        assert_eq!(flow.state, FlowState::Aborted);
        assert!(!flow.applied());
    }

    #[test]
    fn update_flow_q_cancels_from_confirm() {
        let mut flow = UpdateFlow::notes_available("0.3.1", "notes");
        flow.handle_key(KeyCode::Enter);
        assert_eq!(flow.state, FlowState::Confirm);
        flow.handle_key(KeyCode::Char('q'));
        assert_eq!(flow.state, FlowState::Aborted);
        assert!(!flow.applied());
    }

    #[test]
    fn update_flow_ctrl_c_aborts_every_live_step_without_apply() {
        let mut flow = UpdateFlow::notes_available("0.3.1", "notes");
        flow.handle_key_event(ctrl_c_event());
        assert_eq!(flow.state, FlowState::Aborted);
        assert!(!flow.applied());

        let mut flow = UpdateFlow::notes_available("0.3.1", "notes");
        flow.handle_key(KeyCode::Enter);
        assert_eq!(flow.state, FlowState::Confirm);
        flow.handle_key_event(ctrl_c_event());
        assert_eq!(flow.state, FlowState::Aborted);
        assert!(!flow.applied());

        // Terminal states ignore Ctrl+C.
        let mut flow = UpdateFlow::from_info(&up_to_date_info());
        assert_eq!(flow.state, FlowState::Applied);
        flow.handle_key_event(ctrl_c_event());
        assert_eq!(flow.state, FlowState::Applied);
        assert!(!flow.applied());
    }

    #[test]
    fn update_flow_from_info_branches() {
        let flow = UpdateFlow::from_info(&sample_available_info());
        assert_eq!(flow.state, FlowState::Notes);
        assert!(!flow.applied());

        let flow = UpdateFlow::from_info(&up_to_date_info());
        assert_eq!(flow.state, FlowState::Applied);
        assert!(!flow.applied());
        assert_eq!(flow.output(), Some("up to date"));

        let mut flapped = sample_available_info();
        flapped.available = true;
        flapped.version = None;
        let flow = UpdateFlow::from_info(&flapped);
        assert_eq!(flow.state, FlowState::Applied);
        assert!(!flow.applied());
    }

    #[test]
    fn update_flow_enter_on_applied_is_idempotent() {
        let mut flow = UpdateFlow::notes_available("0.3.1", "notes");
        flow.handle_key(KeyCode::Enter);
        flow.handle_key(KeyCode::Enter);
        assert!(flow.applied());
        let output_before = flow.output().map(str::to_string);
        // Extra Enters on Applied are no-ops: same state, same output.
        flow.handle_key(KeyCode::Enter);
        flow.handle_key(KeyCode::Enter);
        assert_eq!(flow.state, FlowState::Applied);
        assert!(flow.applied());
        assert_eq!(flow.output().map(str::to_string), output_before);
    }

    #[test]
    fn update_flow_notes_render_prompts_once() {
        let flow = UpdateFlow::notes_available("0.3.1", "- fixed tray races");
        let content = render_text(&flow);
        assert_eq!(
            content.matches("Enter confirm").count(),
            1,
            "prompt must live in exactly one place, got:\n{content}"
        );
    }

    #[test]
    fn update_flow_confirm_render_without_url_has_single_prompt() {
        let mut flow = UpdateFlow::notes_available("0.3.1", "notes");
        flow.handle_key(KeyCode::Enter);
        assert_eq!(flow.state, FlowState::Confirm);
        let content = render_text(&flow);
        assert_eq!(
            content.matches("Enter confirm").count(),
            1,
            "prompt must live in exactly one place, got:\n{content}"
        );
        assert!(content.contains("install_update"));
    }

    #[test]
    fn update_flow_confirm_render_with_url_links_installer() {
        let mut flow = UpdateFlow::from_info(&sample_available_info());
        flow.handle_key(KeyCode::Enter);
        assert_eq!(flow.state, FlowState::Confirm);
        let content = render_text(&flow);
        assert!(content.contains("https://example.com/mochi-0.3.1.dmg"));
        assert!(content.contains("install_update"));
    }

    #[test]
    fn update_flow_terminal_states_render() {
        let flow = UpdateFlow::from_info(&up_to_date_info());
        assert!(render_text(&flow).contains("up to date"));
        let mut flow = UpdateFlow::notes_available("0.3.1", "notes");
        flow.handle_key(KeyCode::Esc);
        assert!(render_text(&flow).contains("Cancelled"));
    }

    #[test]
    fn update_flow_up_to_date_yields_printable_output() {
        let info = up_to_date_info();
        let flow = UpdateFlow::from_info(&info);
        assert_eq!(flow.state, FlowState::Applied);
        assert!(!flow.applied());
        let output = super::final_output_for_flow(&info, &flow).expect("printable output");
        assert!(
            output.contains("up to date"),
            "up-to-date path must print, got: {output}"
        );
    }

    #[test]
    fn update_flow_final_output_reuses_snapshot_without_refetch() {
        let info = sample_available_info();
        let mut flow = UpdateFlow::from_info(&info);
        flow.handle_key(KeyCode::Enter);
        flow.handle_key(KeyCode::Enter);
        assert!(flow.applied());
        let printed = super::final_output_for_flow(&info, &flow).expect("applied output");
        // Matches the confirmed snapshot even if the live feed flapped.
        assert_eq!(
            printed,
            crate::cli::update::format_apply_output(&info),
            "printed output must reflect the confirmed snapshot"
        );
        assert_eq!(printed, flow.output().expect("confirm-screen output"));
        let mut flapped = info.clone();
        flapped.version = Some("9.9.9".to_string());
        flapped.download_url = Some("https://example.com/other.dmg".to_string());
        assert_ne!(
            printed,
            crate::cli::update::format_apply_output(&flapped),
            "a changed feed must not rewrite the confirmed output"
        );
        // Aborted flows print nothing.
        let mut flow = UpdateFlow::notes_available("0.3.1", "notes");
        flow.handle_key(KeyCode::Esc);
        assert!(super::final_output_for_flow(&info, &flow).is_none());
    }

    #[test]
    fn update_flow_precheck_failure_matches_headless_shape() {
        let (message, code) = super::precheck_failure("request timeout".to_string());
        assert_eq!(message, "request timeout");
        assert!(!message.contains("mochi failed:"));
        assert_eq!(code, crate::cli::tui::EXIT_DOMAIN);
    }

    #[test]
    fn update_flow_confirm_rows_skip_absent_url() {
        let flow = UpdateFlow::notes_available("0.3.1", "notes");
        assert_eq!(
            flow.confirm_rows(),
            vec![
                "0.3.1 available".to_string(),
                "install via the GUI updater (install_update)".to_string(),
            ]
        );
        let flow = UpdateFlow::from_info(&sample_available_info());
        assert_eq!(
            flow.confirm_rows(),
            vec![
                "0.3.1 available".to_string(),
                "https://example.com/mochi-0.3.1.dmg".to_string(),
                "install via the GUI updater (install_update)".to_string(),
            ]
        );
    }
}
