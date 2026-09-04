# CLI TUI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Spec:** `docs/cli-tui/spec.md`
**Branch:** `feature/cli-tui`

**Goal:** Rust-native ratatui TUI for mochi's interactive CLI surfaces (config wizard, update flow, usage dashboard, cost view) with real implementations for the 4 stub commands, preserving plain output for scripting and fixing the Windows console.

**Architecture:** Phase 1 implements the 4 stub commands as plain-output commands reusing the existing store/settings/fetch code. Phase 2 adds the `cli/tui/` state-machine module (gate, screens, restore guards) plus the Windows console attach/alloc. Phase 3 builds the four screens on TestBackend-tested rendering. No new `#[cfg(target_os)]` in `src-tauri/src/core/`.

**Tech Stack:** Rust, clap 4.5 (existing), ratatui + crossterm (new), windows-sys 0.59 + `Win32_System_Console` feature (existing dep, extended), TestBackend snapshot tests.

## Global Constraints

- Branch: `feature/cli-tui`; never commit directly to `main`.
- Conventional commits per `docs/agent-rules/commit-messages.md`; subject < 50 chars; no agent footers.
- After every task: `pnpm lint && pnpm format:check && pnpm test` green; `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all-targets` green before commit.
- Exit contract: `0` success, `1` domain failure (human stderr message, no stack), `2` usage error — every new command and screen honors it.
- Cookie/token values never rendered, logged, or snapshotted; masked input only.
- No new `#[cfg(target_os)]` branches in `src-tauri/src/core/` (CA-07); Windows code lives in `src-tauri/src/cli/windows_console.rs`.
- Rust modules under 300 lines when practical.

---

### Task 1: Real `status` command (plain output)

**Files:**
- Create: `src-tauri/src/cli/status.rs`
- Modify: `src-tauri/src/lib.rs` (`run_cli` match arm, lines ~207-234)
- Modify: `src-tauri/src/cli/mod.rs` (add `pub mod status;`)

**Interfaces:**
- Consumes: `cli_usage_states(provider, false)` (same helper `run_cli` uses for `Usage`); `provider_display_name` from `crate::tray`.
- Produces: `format_status_text(states: &[ProviderUsageState]) -> String` — one line per provider: `<label> <state>` (fresh `64%`, `credentials missing`, `update available`, error message passthrough). `Command::Status` arm prints it, exit 0.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
    use crate::core::usage_state::{ProviderUsageState, ProviderUsageStateKind};

    fn fresh_claude() -> ProviderUsageState {
        let snapshot = UsageSnapshot::new(
            ProviderId::Claude,
            UsageWindow::new("Session", 64.0, None),
            None,
            "2026-06-04T12:00:00Z",
            "test",
        );
        ProviderUsageState::fresh(snapshot)
    }

    #[test]
    fn status_line_shows_label_and_percent() {
        let output = format_status_text(&[fresh_claude()]);
        assert!(output.contains("Claude"));
        assert!(output.contains("64%"));
    }

    #[test]
    fn status_line_marks_missing_credentials() {
        let state = ProviderUsageState {
            provider: ProviderId::Zai,
            kind: ProviderUsageStateKind::MissingCredentials,
            snapshot: None,
            message: None,
        };
        let output = format_status_text(&[state]);
        assert!(output.contains("credentials missing"));
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml status::`
Expected: FAIL — `format_status_text` not defined (check `ProviderUsageState` struct shape first in `src-tauri/src/core/usage_state.rs`; adapt field names).

- [ ] **Step 3: Implement `format_status_text` in `cli/status.rs`**

```rust
use crate::core::usage_state::{ProviderUsageState, ProviderUsageStateKind};
use crate::tray::provider_display_name;

pub fn format_status_text(states: &[ProviderUsageState]) -> String {
    if states.is_empty() {
        return "No providers configured.".to_string();
    }
    states
        .iter()
        .map(|state| match (&state.kind, state.snapshot.as_ref()) {
            (_, Some(snapshot)) => format!(
                "{} {}%{}",
                provider_display_name(snapshot.provider),
                snapshot.primary.used_percent.round() as u8,
                state
                    .message
                    .as_ref()
                    .map(|message| format!(" ({message})"))
                    .unwrap_or_default()
            ),
            (ProviderUsageStateKind::MissingCredentials, None) => {
                format!("{} credentials missing", provider_display_name(state.provider))
            }
            (_, None) => format!(
                "{} {}",
                provider_display_name(state.provider),
                state.message.as_deref().unwrap_or("no data")
            ),
        })
        .collect::<Vec<_>>()
        .join("\n")
}
```

- [ ] **Step 4: Wire the `Command::Status` arm in `run_cli`**

```rust
Command::Status { provider } => {
    let states = cli_usage_states(provider.as_deref(), false)?;
    println!("{}", cli::status::format_status_text(&states));
}
```

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml status::`
Expected: PASS.

- [ ] **Step 6: Rust validation**

Run: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-targets`
Expected: green.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/cli/status.rs src-tauri/src/cli/mod.rs src-tauri/src/lib.rs
git commit -m "feat(cli): implement status command"
```

---

### Task 2: Real `cost` command (plain output)

**Files:**
- Create: `src-tauri/src/cli/cost.rs`
- Modify: `src-tauri/src/lib.rs` (`run_cli` match arm)
- Modify: `src-tauri/src/cli/mod.rs` (add `pub mod cost;`)

**Interfaces:**
- Consumes: `SqliteUsageRepository` history read path (same repository `initialize_usage_store` opens; find the history query in `src-tauri/src/core/` — e.g. cost snapshots by day); `ProviderId` display names.
- Produces: `format_cost_text(entries: &[CostEntry], days: u16) -> String` — one line per provider: `<label> $<used> / $<limit> <currency> (<period>)`; empty → `No cost data in the last {days} days.` `Command::Cost` arm prints it, exit 0.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn cost_line_shows_used_vs_limit() {
    let entries = vec![CostEntry {
        provider: ProviderId::CommandCode,
        used: 7.54,
        limit: 71.93,
        currency_code: "USD".to_string(),
        period: "billing-period".to_string(),
    }];
    let output = format_cost_text(&entries, 30);
    assert!(output.contains("$7.54 / $71.93"));
    assert!(output.contains("USD"));
}

#[test]
fn cost_empty_reports_range() {
    assert!(format_cost_text(&[], 30).contains("30 days"));
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml cost::`
Expected: FAIL — `CostEntry`/`format_cost_text` not defined.

- [ ] **Step 3: Implement `cli/cost.rs`**

Define `CostEntry { provider: ProviderId, used: f64, limit: f64, currency_code: String, period: String }`, `format_cost_text` per the test, and `load_cost_entries(days: u16) -> ProviderResult<Vec<CostEntry>>` reading the SQLite history through the existing repository (open the same `usage.sqlite3` path the CLI helpers use — find `cli_usage_states` in `lib.rs` lines ~235+ and reuse its store-opening code; TDD the pure formatter, integration-test the loader against a temp-dir DB if the repository supports it, else manual evidence).

- [ ] **Step 4: Wire the `Command::Cost` arm in `run_cli`**

```rust
Command::Cost { provider, days } => {
    let entries = cli::cost::load_cost_entries(days)?;
    let entries: Vec<_> = match provider.as_deref() {
        Some(name) => entries.into_iter().filter(|e| e.provider.as_str() == name).collect(),
        None => entries,
    };
    println!("{}", cli::cost::format_cost_text(&entries, days));
}
```

(Check `ProviderId::as_str` exists — Task 5/6 added it; adapt the filter to the real accessor.)

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml cost::`
Expected: PASS.

- [ ] **Step 6: Rust validation** (same command as Task 1 Step 6)

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/cli/cost.rs src-tauri/src/cli/mod.rs src-tauri/src/lib.rs
git commit -m "feat(cli): implement cost command"
```

---

### Task 3: Real `config` command (get/set/list)

**Files:**
- Create: `src-tauri/src/cli/config.rs`
- Modify: `src-tauri/src/lib.rs` (`run_cli` match arm)
- Modify: `src-tauri/src/cli/mod.rs` (add `pub mod config;`)

**Interfaces:**
- Consumes: the settings file path + load/save used by `SettingsState` (find in `src-tauri/src/settings/storage.rs`; CLI has no `AppHandle`, so reuse the path helper + serde directly — do NOT duplicate the schema).
- Produces: `config list` prints `key = value` lines (provider allowlist: `update_channel`, `enabled_providers`, per-provider `cookie_source` only — never secret values; secrets print `<set>`/`<unset>`); `config get <key>` prints the value or exits 1 with `unknown key: <key>`; `config set <key> <value>` validates through the real settings deserialization, writes the file, prints `key = value`. Unknown key → exit 1; invalid value → exit 1 with the serde error.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn config_get_unknown_key_errors() {
    let dir = tempdir().expect("tempdir");
    let err = run_config_get(&dir, "nope").expect_err("unknown key");
    assert!(err.contains("unknown key"));
}

#[test]
fn config_set_round_trips() {
    let dir = tempdir().expect("tempdir");
    run_config_set(&dir, "update_channel", "stable").expect("set");
    assert_eq!(run_config_get(&dir, "update_channel").expect("get"), "stable");
}

#[test]
fn config_never_prints_secrets() {
    let dir = tempdir().expect("tempdir");
    run_config_set(&dir, "update_channel", "stable").expect("set");
    let list = run_config_list(&dir).expect("list");
    assert!(!list.contains("session_token"));
    assert!(!list.contains("api_key"));
}
```

(Use `tempfile::tempdir` — check dev-dependencies; if absent, `std::env::temp_dir` + unique subdir like the commandcode strategy test does.)

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml config::`
Expected: FAIL — functions not defined.

- [ ] **Step 3: Implement `cli/config.rs`**

Pure functions `run_config_list(dir)`, `run_config_get(dir, key)`, `run_config_set(dir, key, value)` operating on the settings file in `dir`; the `run_cli` arm resolves `dir` from the real settings path helper. Key allowlist is a `const`: `["update_channel", "enabled_providers"]` + per-provider cookie-source keys derived from the provider registry (no hardcoded provider list). Secrets masked in list output.

- [ ] **Step 4: Wire the `Command::Config` arm**

```rust
Command::Config { key, value } => match (key, value) {
    (None, None) => println!("{}", cli::config::run_config_list(&settings_dir()?)?),
    (Some(key), None) => println!("{}", cli::config::run_config_get(&settings_dir()?, &key)?),
    (Some(key), Some(value)) => {
        println!("{}", cli::config::run_config_set(&settings_dir()?, &key, &value)?)
    }
    (None, Some(_)) => {
        eprintln!("usage: mochi config [get <key> | set <key> <value>]");
        std::process::exit(2);
    }
};
```

(`settings_dir()` = the existing helper the CLI path can reuse — find it; if none exists, extract one from `SettingsState` path logic without changing GUI behavior.)

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml config::`
Expected: PASS.

- [ ] **Step 6: Rust validation** (same as Task 1 Step 6)

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/cli/config.rs src-tauri/src/cli/mod.rs src-tauri/src/lib.rs
git commit -m "feat(cli): implement config command"
```

---

### Task 4: Real `update` command (check/apply, plain output)

**Files:**
- Create: `src-tauri/src/cli/update.rs`
- Modify: `src-tauri/src/lib.rs` (`run_cli` match arm)
- Modify: `src-tauri/src/cli/mod.rs` (add `pub mod update;`)

**Interfaces:**
- Consumes: the stable updater feed check (same endpoint/logic `updater::check_for_update` uses — extract a non-GUI `check_stable_update() -> ProviderResult<Option<UpdateInfo { version, notes }>>` if one doesn't exist; do not duplicate feed parsing).
- Produces: `mochi update check` prints `up to date` or `<version> available` + notes; `mochi update apply` requires `--confirm` (without it: usage error exit 2), then applies and prints `updated to <version>`; any other action → exit 2 with usage.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn update_rejects_unknown_action() {
    let err = run_update_action("frobnicate", false).expect_err("usage");
    assert!(err.contains("usage"));
}

#[test]
fn update_apply_requires_confirm() {
    let err = run_update_action("apply", false).expect_err("confirm");
    assert!(err.contains("--confirm"));
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml cli::update`
Expected: FAIL — `run_update_action` not defined.

- [ ] **Step 3: Implement `cli/update.rs`**

`run_update_action(action: &str, confirm: bool) -> Result<String, String>` dispatching `check`/`apply`; `apply` without confirm returns `Err("refusing to apply without --confirm")`. Live check/apply call the extracted updater functions; note: the `Command::Update { action }` clap shape needs a `--confirm` flag added in `cli/mod.rs`.

- [ ] **Step 4: Wire the arm + `--confirm` flag**

```rust
Command::Update { action, confirm } => match cli::update::run_update_action(&action, confirm) {
    Ok(output) => println!("{output}"),
    Err(message) => {
        eprintln!("{message}");
        std::process::exit(1);
    }
};
```

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml cli::update`
Expected: PASS.

- [ ] **Step 6: Rust validation** (same as Task 1 Step 6)

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/cli/update.rs src-tauri/src/cli/mod.rs src-tauri/src/lib.rs
git commit -m "feat(cli): implement update command"
```

---

### Task 5: TUI shell — gate, exit contract, restore guards, deps

**Files:**
- Modify: `src-tauri/Cargo.toml` (add `ratatui`, `crossterm`; extend `windows-sys` features with `Win32_System_Console`)
- Create: `src-tauri/src/cli/tui/mod.rs` (re-exports + `should_use_tui`)
- Create: `src-tauri/src/cli/tui/guard.rs` (terminal restore guard + panic hook)
- Modify: `src-tauri/src/cli/mod.rs` (add `pub mod tui;`)

**Interfaces:**
- Consumes: `std::io::IsTerminal`; clap matches (whether `--json`/format flags present — gate takes explicit bools so it stays testable).
- Produces: `should_use_tui(stdin_tty: bool, stdout_tty: bool, machine_output: bool) -> bool` (true only when all TTY and no machine flag); `TuiGuard::enter() -> io::Result<TuiGuard>` enabling raw mode + alternate screen, restoring both on drop; `install_panic_hook()` chaining the default hook after restoring the terminal. Exit-code constants `EXIT_OK/EXIT_DOMAIN/EXIT_USAGE = 0/1/2`.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn tui_gate_requires_tty_and_human_output() {
    assert!(should_use_tui(true, true, false));
    assert!(!should_use_tui(false, true, false));
    assert!(!should_use_tui(true, false, false));
    assert!(!should_use_tui(true, true, true));
}

#[test]
fn guard_restores_terminal_on_drop() {
    // Gated: only meaningful on a real TTY; assert construction contract instead:
    // enter() on non-TTY returns Err (crossterm errors without a terminal).
    assert!(TuiGuard::enter().is_err() || std::io::stdin().is_terminal());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml cli::tui`
Expected: FAIL — module/items not defined.

- [ ] **Step 3: Implement gate + guard**

`should_use_tui` per the test. `TuiGuard` wraps `crossterm::terminal::{enable_raw_mode, EnterAlternateScreen}` on enter and disables/leaves on drop; `install_panic_hook` takes the default hook, installs one that drops a guard-equivalent restore then calls the original. Keep each file under 300 lines (this one ~80).

- [ ] **Step 4: Add dependencies**

```toml
ratatui = "0.30"
crossterm = "0.29"
```

(Verify latest compatible versions with `cargo add --dry-run` semantics at implementation time — pin what resolves; record in the report. Extend the existing `windows-sys` features list with `"Win32_System_Console"`.)

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml cli::tui`
Expected: PASS.

- [ ] **Step 6: Rust validation** (same as Task 1 Step 6)

- [ ] **Step 7: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/cli/tui/ src-tauri/src/cli/mod.rs
git commit -m "feat(cli): add tui shell and gate"
```

---

### Task 6: Windows console attach/alloc

**Files:**
- Create: `src-tauri/src/cli/windows_console.rs`
- Modify: `src-tauri/src/lib.rs` (call `cli::windows_console::ensure_console()` at the top of `run_cli`)
- Modify: `src-tauri/src/cli/mod.rs` (add `#[cfg(windows)] pub mod windows_console;`)

**Interfaces:**
- Consumes: `windows_sys::Win32::System::Console::{AttachConsole, AllocConsole, ATTACH_PARENT_PROCESS}` (feature added in Task 5).
- Produces: `#[cfg(windows)] pub fn ensure_console()` — `AttachConsole(ATTACH_PARENT_PROCESS)`; on failure, `AllocConsole()`; both failures → return silently (plain output may vanish, TUI reports the error — never panic). `#[cfg(not(windows))] pub fn ensure_console()` — no-op. Pure helper `attach_or_alloc(attached: bool, allocated: bool)` is unit-tested for the decision table; the real syscalls are manual-QA (Windows Terminal checklist in Task 10).

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(windows)]
#[test]
fn attach_success_skips_alloc() {
    assert_eq!(attach_or_alloc(true, false), ConsoleAction::Attached);
}

#[cfg(windows)]
#[test]
fn alloc_fallback_when_no_parent() {
    assert_eq!(attach_or_alloc(false, true), ConsoleAction::Allocated);
}
```

(Also provide the `not(windows)` no-op compile test: `ensure_console()` links on Linux CI.)

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml windows_console`
Expected: FAIL — module not defined (on Linux CI the `#[cfg(windows)]` tests compile out; the no-op must still compile — gate the enum helper `#[cfg(any())]`-free so logic tests run everywhere: put `attach_or_alloc` + `ConsoleAction` outside the cfg gate, only the syscall wrappers inside).

- [ ] **Step 3: Implement**

Decision-table helper ungated; `ensure_console()` cfg-split. Call it as the first line of `run_cli` (all platforms; no-op off Windows).

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml windows_console`
Expected: PASS.

- [ ] **Step 5: Rust validation** (same as Task 1 Step 6)

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/cli/windows_console.rs src-tauri/src/cli/mod.rs src-tauri/src/lib.rs
git commit -m "fix(cli): attach windows console"
```

---

### Task 7: Config wizard screen

**Files:**
- Create: `src-tauri/src/cli/tui/screens.rs` (screen enum + shared list-navigation state)
- Create: `src-tauri/src/cli/tui/config_wizard.rs` (wizard state machine + rendering)
- Modify: `src-tauri/src/lib.rs` (route `Command::Config` without key/value to the wizard when `should_use_tui()`)

**Interfaces:**
- Consumes: Task 3's `run_config_list/get/set` + provider registry (enabled_providers, cookie sources); `resolve_session_cookie`-compatible manual paste path (masked input, never logged).
- Produces: `ConfigWizard { step, providers, edits }` with `next/back/confirm` transitions; `render(frame, &wizard)` drawing via ratatui `TestBackend`-compatible code; finishing confirm calls the same setters as `run_config_set` and returns the reviewed values. Cancelling (Esc/q) returns `Ok(None)` — settings untouched (CA-01 abort case).

- [ ] **Step 1: Write the failing TestBackend tests**

```rust
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
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml config_wizard`
Expected: FAIL — types not defined.

- [ ] **Step 3: Implement the wizard**

State machine: `ProviderList → ProviderDetail(cookie source radio + masked paste) → Review → Done/Cancelled`. Keep rendering functions pure over `&Wizard`. Masked paste: `tui-input`-free — track a `String` buffer, render `•`.repeat(len). Reuse Task 3 setters on confirm.

- [ ] **Step 4: Route `Command::Config` (no args) to the wizard under the gate**

```rust
Command::Config { key: None, value: None } if should_use_tui() => {
    cli::tui::config_wizard::run_config_wizard()
}
```

(`should_use_tui()` reads real TTY state + false machine flag; the pure helper from Task 5 stays unit-tested.)

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml config_wizard`
Expected: PASS.

- [ ] **Step 6: Rust validation** (same as Task 1 Step 6)

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/cli/tui/ src-tauri/src/lib.rs
git commit -m "feat(cli): add config wizard screen"
```

---

### Task 8: Update flow screen

**Files:**
- Create: `src-tauri/src/cli/tui/update_flow.rs`
- Modify: `src-tauri/src/lib.rs` (route `Command::Update` to the flow screen under the gate)

**Interfaces:**
- Consumes: Task 4's `run_update_action("check", ...)` result type + updater `UpdateInfo { version, notes }`.
- Produces: `UpdateFlow { state: Check/Notes/Confirm/Applied }`; notes preview rendered from the stable feed; confirm only on explicit Enter on the confirm button, Esc cancels with no mutation (CA-03). `run_update_flow() -> anyhow::Result<()>` drives the event loop with the Task 5 guard.

- [ ] **Step 1: Write the failing TestBackend tests**

```rust
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
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml update_flow`
Expected: FAIL.

- [ ] **Step 3: Implement the flow** (state machine + rendering + event loop with guard; apply path calls Task 4's apply with confirm=true internally).

- [ ] **Step 4: Route under the gate** (same pattern as Task 7 Step 4).

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml update_flow`
Expected: PASS.

- [ ] **Step 6: Rust validation** (same as Task 1 Step 6)

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/cli/tui/ src-tauri/src/lib.rs
git commit -m "feat(cli): add update flow screen"
```

---

### Task 9: Usage dashboard + cost view screens

**Files:**
- Create: `src-tauri/src/cli/tui/usage_dashboard.rs`
- Create: `src-tauri/src/cli/tui/cost_view.rs`
- Modify: `src-tauri/src/lib.rs` (route `Command::Usage`/`Status`/`Cost` under the gate)

**Interfaces:**
- Consumes: Task 1's states + Task 2's `CostEntry` list; widget label strings (`5 hours`, `Weekly`, `Billing period` + money line — copy the exact literals from `provider-cost-section.tsx`, do not invent variants).
- Produces: `render_usage_dashboard(frame, &states)` table (provider, primary %, reset) + `render_cost_view(frame, &entries)` money lines; read-only screens (q/Esc exits, r refreshes). TestBackend snapshot tests pin the labels (CA-04).

- [ ] **Step 1: Write the failing TestBackend tests**

```rust
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
fn cost_view_shows_money_line() {
    let backend = TestBackend::new(60, 20);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal
        .draw(|frame| render_cost_view(frame, &fixture_costs()))
        .expect("draw");
    assert!(terminal.backend().to_string().contains("$7.54 / $71.93"));
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml "usage_dashboard"`
Expected: FAIL.

- [ ] **Step 3: Implement both screens** (read-only; refresh key re-reads the store).

- [ ] **Step 4: Route under the gate.**

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml "usage_dashboard"`
Expected: PASS.

- [ ] **Step 6: Rust validation** (same as Task 1 Step 6)

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/cli/tui/ src-tauri/src/lib.rs
git commit -m "feat(cli): add usage and cost screens"
```

---

### Task 10: QA evidence + docs

**Files:**
- Create: `docs/qa/cli-tui-evidence.md`
- Modify: `docs/releasing.md` (one paragraph: CLI/TUI surface + exit contract) — only if the file has a CLI section; otherwise skip.

**Interfaces:**
- Consumes: all prior tasks.
- Produces: evidence doc with per-surface status (PASS/manual-checklist) for GNOME terminal, macOS Terminal, Windows Terminal; exit-code matrix proof (`; echo $?` for 0/1/2 paths); TTY-fallback proof (piped + `--json` runs); masked-secret proof (screenshot description or redacted log excerpt).

- [ ] **Step 1: Run the exit-code matrix**

Run: `mochi status --bad-flag; echo $?` (expect 2), plus a failing-domain path (expect 1), plus a success (expect 0). Record outputs.

- [ ] **Step 2: Run the TTY-fallback proof**

Run: `mochi status | cat` and `mochi usage --json | head -c 200` — confirm plain output, no TUI escape codes. Record.

- [ ] **Step 3: Write `docs/qa/cli-tui-evidence.md`**

Date, machine/session info, exit-code table, fallback proof, per-screen manual checklist with PASS/PENDING-OWNER marks and repro steps. Honest about what was scripted vs eyeballed.

- [ ] **Step 4: Commit**

```bash
git add docs/qa/cli-tui-evidence.md docs/releasing.md
git commit -m "docs(qa): record cli tui evidence"
```
