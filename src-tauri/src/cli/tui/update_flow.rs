//! Update flow TUI: check → notes preview → confirm → applied/aborted.
//! Honest semantics (Task 4): the headless CLI never replaces the binary,
//! so Applied reports the version, the installer URL, and the GUI-install
//! path — never "updated to". Esc cancels with no mutation (CA-03).

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, Paragraph},
    Frame, Terminal,
};

use super::{install_panic_hook, TuiGuard};
use crate::cli::update::{format_apply_output, run_update_action};
use crate::updater::{check_stable_update, UpdateInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlowState {
    #[default]
    Check,
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
            state: FlowState::Check,
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
        if matches!(
            self.state,
            FlowState::Check | FlowState::Notes | FlowState::Confirm
        ) {
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
            FlowState::Check => match key {
                KeyCode::Enter if self.version.is_some() => self.state = FlowState::Notes,
                KeyCode::Enter => {
                    self.output = Some("up to date".to_string());
                    self.state = FlowState::Applied;
                }
                KeyCode::Esc => self.abort(),
                _ => {}
            },
            FlowState::Notes => match key {
                KeyCode::Enter => self.state = FlowState::Confirm,
                KeyCode::Esc => self.abort(),
                _ => {}
            },
            // Confirm ONLY on explicit Enter; anything else leaves the flow
            // put (Esc aborts with no mutation).
            FlowState::Confirm => match key {
                KeyCode::Enter => {
                    self.output = Some(format_apply_output(&self.info()));
                    self.confirmed = true;
                    self.state = FlowState::Applied;
                }
                KeyCode::Esc => self.abort(),
                _ => {}
            },
            FlowState::Applied | FlowState::Aborted => {}
        }
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
        FlowState::Check => (
            "Mochi update".to_string(),
            vec!["Checking for updates…".to_string()],
            "Enter continue · Esc cancel",
        ),
        FlowState::Notes => {
            let mut rows: Vec<String> = flow.notes.lines().map(str::to_string).collect();
            if rows.is_empty() {
                rows.push("(no release notes)".to_string());
            }
            rows.push(String::new());
            rows.push("Enter confirm · Esc cancel".to_string());
            (
                format!("Mochi update — {version} available"),
                rows,
                "Enter confirm · Esc cancel",
            )
        }
        FlowState::Confirm => (
            format!("Mochi update — confirm {version}"),
            vec![
                format!("{version} available"),
                flow.download_url.clone().unwrap_or_default(),
                "install via the GUI updater (install_update)".to_string(),
                String::new(),
                "Enter confirm · Esc cancel".to_string(),
            ],
            "Enter confirm · Esc cancel (no change until Enter)",
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

/// Run the fullscreen update event loop. Returns after the terminal is
/// restored; the live apply runs only after explicit in-TUI confirm, through
/// the same code path as the plain `update apply --confirm` command.
pub fn run_update_flow() -> anyhow::Result<()> {
    // Install LAST so terminal restore runs before the diagnostics logger.
    install_panic_hook();
    let info = check_stable_update().map_err(|message| anyhow::anyhow!(message))?;
    let mut flow = UpdateFlow::from_info(&info);
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
    if flow.applied() {
        match run_update_action("apply", true) {
            Ok(output) => println!("{output}"),
            Err(message) => return Err(anyhow::anyhow!(message)),
        }
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
}
