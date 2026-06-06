# Linux Native Window Controls Unstable Experiments Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a sequence of isolated unstable Linux diagnostic releases that prove which Tauri window lifecycle behavior breaks Ubuntu Wayland native window-control hit regions, then implement only the proven fix.

**Architecture:** Add a small Rust window-policy layer that selects a Linux window experiment at build time via `MOCHI_LINUX_WINDOW_EXPERIMENT`, logs the selected policy into diagnostics, and changes exactly one window lifecycle variable per unstable try. The experiments are published through the existing unstable release workflow and tested on Ubuntu Wayland installed packages, not only `tauri dev`.

**Tech Stack:** Tauri v2 Rust backend, React 19 + Vite 8 frontend, Tailwind CSS 4 app-window shells, Vitest source tests, Rust unit tests, GitHub Actions unstable release workflow.

---

## Experiment Order

Run these in order. Do not skip to later variants until the earlier variant has an installed Ubuntu Wayland result.

1. `baseline-sequenced-logs`: no behavior change; add lifecycle diagnostics and widget/settings background parity.
2. `on-demand-visible`: Linux settings/about/update/widget windows are created only in response to user action and are created visible.
3. `on-demand-hidden`: only if `on-demand-visible` fixes the bug; same as Variant 2 but created hidden then shown, to isolate visible creation from startup precreation.
4. `builder-size-only`: only if `on-demand-visible` does not fix the bug; remove first-open `set_size()` / `set_min_size()` mutation and use builder sizes.
5. `show-focus-only`: only if Variant 4 does not fix the bug; keep builder-only sizes and use `show -> set_focus`.
6. `unminimize-show-focus`: only if Variant 5 does not fix the bug; keep builder-only sizes and use `unminimize -> show -> set_focus`.

Every unstable build must include its variant in:

- `mochi diagnostics --bundle`
- GitHub unstable release body
- update feed release notes

---

## File Structure

- Create: `src-tauri/src/window_policy.rs`
  - Owns the experiment enum, compile-time env parsing, per-platform policy decisions, and pure tests.
- Modify: `src-tauri/src/lib.rs`
  - Registers `window_policy` module and conditionally skips Linux startup precreation for experiments that require on-demand creation.
- Modify: `src-tauri/src/tray/panel.rs`
  - Routes settings/about/update window creation through the policy layer and logs lifecycle steps.
- Modify: `src-tauri/src/widget/commands.rs`
  - Routes widget creation/show through the policy layer and logs lifecycle steps.
- Modify: `src-tauri/src/diagnostics/state.rs`
  - Adds structured lifecycle diagnostic records.
- Modify: `src-tauri/src/diagnostics/report.rs`
  - Includes lifecycle records in `mochi diagnostics --bundle`.
- Modify: `src-tauri/tauri.conf.json`
  - Removes or keeps the configured widget window per selected policy; baseline remains unchanged until the on-demand experiment.
- Modify: `src/components/layout/root-component.tsx`
  - Treats widget as app-window shell for document/body attributes or mirrors app-window attributes explicitly.
- Modify: `src/styles/index.css`
  - Adds widget document/body selectors to app-window background rules.
- Modify: `src/styles/app-window-platform.test.ts`
  - Asserts widget shell selectors match app-window shell selectors.
- Modify: `src/lib/tauri/native-window-controls.test.ts`
  - Adds source tests for experiment policy and forbidden post-build decoration mutation.
- Modify: `.github/workflows/release-unstable.yml`
  - Adds workflow input/env support for `MOCHI_LINUX_WINDOW_EXPERIMENT` and release body labelling.
- Modify: `docs/releasing.md`
  - Documents how to publish an unstable diagnostic build with a named experiment.
- Create: `docs/qa/linux-native-window-controls-experiments.md`
  - Manual Ubuntu Wayland test script and result table.

---

## Task 1: Add Experiment Policy Skeleton

**Files:**

- Create: `src-tauri/src/window_policy.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/window_policy.rs`

- [ ] **Step 1: Write the failing Rust tests**

Create `src-tauri/src/window_policy.rs` with only the tests and type names first:

```rust
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
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml window_policy
```

Expected: FAIL with unresolved types such as `LinuxWindowExperiment`.

- [ ] **Step 3: Implement the policy types**

Replace `src-tauri/src/window_policy.rs` with:

```rust
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
```

Modify `src-tauri/src/lib.rs` near the module declarations:

```rust
pub mod window_policy;
```

- [ ] **Step 4: Run the test to verify it passes**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml window_policy
```

Expected: PASS with 4 `window_policy` tests.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/window_policy.rs src-tauri/src/lib.rs
git commit -m "test: add linux window experiment policy"
```

---

## Task 2: Add Sequenced Window Lifecycle Diagnostics

**Files:**

- Modify: `src-tauri/src/diagnostics/state.rs`
- Modify: `src-tauri/src/diagnostics/report.rs`
- Modify: `src-tauri/src/tray/panel.rs`
- Modify: `src-tauri/src/widget/commands.rs`
- Test: `src-tauri/src/diagnostics/state.rs`

- [ ] **Step 1: Write failing diagnostics tests**

In `src-tauri/src/diagnostics/state.rs`, add this test inside the existing `#[cfg(test)]` module:

```rust
#[test]
fn window_lifecycle_detail_includes_experiment_and_sequence() {
    let detail = window_lifecycle_detail(
        "settings",
        "show",
        "baseline-sequenced-logs",
        "startup-precreate",
        "hidden",
        Some((520.0, 560.0)),
        Some((520.0, 560.0)),
    );

    assert!(detail.contains("label=settings"));
    assert!(detail.contains("phase=show"));
    assert!(detail.contains("experiment=baseline-sequenced-logs"));
    assert!(detail.contains("creation=startup-precreate"));
    assert!(detail.contains("initial_visibility=hidden"));
    assert!(detail.contains("outer=520x560"));
    assert!(detail.contains("inner=520x560"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml window_lifecycle_detail
```

Expected: FAIL with `cannot find function window_lifecycle_detail`.

- [ ] **Step 3: Implement lifecycle detail formatting and recording**

In `src-tauri/src/diagnostics/state.rs`, add:

```rust
pub fn format_logical_size(size: Option<(f64, f64)>) -> String {
    size.map(|(width, height)| format!("{width:.0}x{height:.0}"))
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn window_lifecycle_detail(
    label: &str,
    phase: &str,
    experiment: &str,
    creation: &str,
    initial_visibility: &str,
    outer_size: Option<(f64, f64)>,
    inner_size: Option<(f64, f64)>,
) -> String {
    format!(
        "label={label} phase={phase} experiment={experiment} creation={creation} initial_visibility={initial_visibility} outer={} inner={}",
        format_logical_size(outer_size),
        format_logical_size(inner_size)
    )
}
```

Add a method to `DiagnosticsState`:

```rust
pub fn record_window_lifecycle(
    &self,
    label: &str,
    phase: &str,
    experiment: &str,
    creation: &str,
    initial_visibility: &str,
    outer_size: Option<(f64, f64)>,
    inner_size: Option<(f64, f64)>,
) {
    let detail = window_lifecycle_detail(
        label,
        phase,
        experiment,
        creation,
        initial_visibility,
        outer_size,
        inner_size,
    );
    self.push("window.lifecycle", detail.clone());
    super::log::log_line("window.lifecycle", &detail);
}
```

If the state currently has no generic `push` helper, add the same append logic used by `record_window_event()` and keep the `log_line()` call.

- [ ] **Step 4: Add call sites for baseline logging**

In `src-tauri/src/tray/panel.rs`, add helper functions:

```rust
fn logical_outer_size(window: &WebviewWindow) -> Option<(f64, f64)> {
    let scale = window.scale_factor().ok()?;
    let size = window.outer_size().ok()?;
    Some((f64::from(size.width) / scale, f64::from(size.height) / scale))
}

fn logical_inner_size(window: &WebviewWindow) -> Option<(f64, f64)> {
    let scale = window.scale_factor().ok()?;
    let size = window.inner_size().ok()?;
    Some((f64::from(size.width) / scale, f64::from(size.height) / scale))
}

fn record_app_window_lifecycle(window: &WebviewWindow, phase: &str, creation: &str, initial_visibility: &str) {
    if let Some(state) = window
        .app_handle()
        .try_state::<crate::diagnostics::DiagnosticsState>()
    {
        let experiment = crate::window_policy::active_linux_window_experiment().name();
        state.record_window_lifecycle(
            SETTINGS_WINDOW_LABEL,
            phase,
            experiment,
            creation,
            initial_visibility,
            logical_outer_size(window),
            logical_inner_size(window),
        );
    }
}
```

Call it after settings creation, before/after `set_size`, before/after `show`, after `unminimize`, and after `set_focus`:

```rust
record_app_window_lifecycle(window, "created", "startup-precreate", "hidden");
record_app_window_lifecycle(&window, "before-set-size", "startup-precreate", "hidden");
record_app_window_lifecycle(&window, "after-set-size", "startup-precreate", "hidden");
record_app_window_lifecycle(&window, "after-show", "startup-precreate", "hidden");
record_app_window_lifecycle(&window, "after-unminimize", "startup-precreate", "hidden");
record_app_window_lifecycle(&window, "after-focus", "startup-precreate", "hidden");
```

In `src-tauri/src/widget/commands.rs`, add equivalent `logical_outer_size`, `logical_inner_size`, and `record_widget_window_lifecycle()` helpers and call them at creation, after show, after unminimize, and after focus.

- [ ] **Step 5: Run tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml window_lifecycle_detail
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/diagnostics/state.rs src-tauri/src/diagnostics/report.rs src-tauri/src/tray/panel.rs src-tauri/src/widget/commands.rs
git commit -m "test: log linux window lifecycle sequencing"
```

---

## Task 3: Fix Widget App-Window Background Parity

**Files:**

- Modify: `src/styles/app-window-platform.test.ts`
- Modify: `src/styles/index.css`
- Modify: `src/components/layout/root-component-state.test.ts`
- Modify: `src/components/layout/root-component.tsx`

- [ ] **Step 1: Write failing CSS tests**

In `src/styles/app-window-platform.test.ts`, add:

```ts
it("applies app-window document backgrounds to widget windows", () => {
  expect(css).toContain("html[data-widget-window]");
  expect(css).toContain("html[data-widget-window] body");
  expect(css).toContain("html[data-widget-window] #root");
  expect(css).toContain('[data-platform="linux"] html[data-widget-window]');
});
```

In `src/components/layout/root-component-state.test.ts`, add:

```ts
it("uses full-height shell classes for widget windows", () => {
  expect(
    shouldUseFullHeightWindowShell({
      isTrayPanelWindow: false,
      isAppWindow: false,
      isWidgetWindow: true,
      platform: "linux",
    }),
  ).toBe(true);
});
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
pnpm test src/styles/app-window-platform.test.ts src/components/layout/root-component-state.test.ts
```

Expected: CSS test FAILS because `html[data-widget-window]` selectors are missing.

- [ ] **Step 3: Update CSS selectors**

In `src/styles/index.css`, update the shell selectors from:

```css
html[data-tray-panel],
html[data-tray-panel] body,
html[data-app-window],
html[data-app-window] body {
  height: 100%;
  background-color: transparent;
}
```

to:

```css
html[data-tray-panel],
html[data-tray-panel] body,
html[data-app-window],
html[data-app-window] body,
html[data-widget-window],
html[data-widget-window] body {
  height: 100%;
  background-color: transparent;
}
```

Update the root selector from:

```css
html[data-tray-panel] #root,
html[data-app-window] #root {
  height: 100%;
  min-height: 0;
}
```

to:

```css
html[data-tray-panel] #root,
html[data-app-window] #root,
html[data-widget-window] #root {
  height: 100%;
  min-height: 0;
}
```

Update dark and Linux opaque selectors the same way:

```css
html[data-tray-panel].dark,
html[data-tray-panel].dark body,
html[data-app-window].dark,
html[data-app-window].dark body,
html[data-widget-window].dark,
html[data-widget-window].dark body {
  background-color: transparent;
}

[data-platform="linux"] html[data-tray-panel],
[data-platform="linux"] html[data-tray-panel] body,
[data-platform="linux"] html[data-app-window],
[data-platform="linux"] html[data-app-window] body,
[data-platform="linux"] html[data-widget-window],
[data-platform="linux"] html[data-widget-window] body {
  background-color: light-dark(#fafafa, #242424);
}
```

- [ ] **Step 4: Run tests**

Run:

```bash
pnpm test src/styles/app-window-platform.test.ts src/components/layout/root-component-state.test.ts
pnpm format:check
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/styles/index.css src/styles/app-window-platform.test.ts src/components/layout/root-component-state.test.ts src/components/layout/root-component.tsx
git commit -m "fix(widget): match app window background shell"
```

---

## Task 4: Wire Unstable Workflow Experiment Input

**Files:**

- Modify: `.github/workflows/release-unstable.yml`
- Modify: `docs/releasing.md`
- Test: `scripts/release/workflow-updater.test.mjs`

- [ ] **Step 1: Write failing workflow test**

In `scripts/release/workflow-updater.test.mjs`, add:

```js
it("passes linux window experiment into unstable builds and release notes", () => {
  const workflow = readFileSync(".github/workflows/release-unstable.yml", "utf8");

  expect(workflow).toContain("linux_window_experiment");
  expect(workflow).toContain("MOCHI_LINUX_WINDOW_EXPERIMENT");
  expect(workflow).toContain("Linux window experiment");
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
pnpm test scripts/release/workflow-updater.test.mjs
```

Expected: FAIL because workflow does not expose the experiment input.

- [ ] **Step 3: Add workflow dispatch input**

In `.github/workflows/release-unstable.yml`, change:

```yaml
on:
  push:
    branches: [main]
  workflow_dispatch:
```

to:

```yaml
on:
  push:
    branches: [main]
  workflow_dispatch:
    inputs:
      linux_window_experiment:
        description: Linux window experiment to compile into the unstable build
        required: false
        default: baseline-sequenced-logs
        type: choice
        options:
          - baseline-sequenced-logs
          - on-demand-visible
          - on-demand-hidden
          - builder-size-only
          - show-focus-only
          - unminimize-show-focus
```

In the `Build unstable Tauri app` step env block, add:

```yaml
MOCHI_LINUX_WINDOW_EXPERIMENT: ${{ github.event.inputs.linux_window_experiment || 'baseline-sequenced-logs' }}
```

In the release body under `## Mochi Unstable`, add:

```markdown
Linux window experiment: `${{ github.event.inputs.linux_window_experiment || 'baseline-sequenced-logs' }}`
```

In the updater feed `release-notes.md`, write:

```bash
cat > release-notes.md <<EOF
### What's changed
- Linux window experiment: \`${{ github.event.inputs.linux_window_experiment || 'baseline-sequenced-logs' }}\`.
EOF
```

- [ ] **Step 4: Document unstable diagnostic publication**

In `docs/releasing.md`, add:

````md
## Linux Window-Control Diagnostic Unstable Builds

Use workflow dispatch on `release-unstable.yml` with `linux_window_experiment` set to one of:

- `baseline-sequenced-logs`
- `on-demand-visible`
- `on-demand-hidden`
- `builder-size-only`
- `show-focus-only`
- `unminimize-show-focus`

Install with the unstable installer:

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-linux.sh | bash -s -- -i
```
````

After each diagnostic build, run:

```bash
mochi diagnostics --bundle
```

Attach the bundle and record the result in `docs/qa/linux-native-window-controls-experiments.md`.

````

- [ ] **Step 5: Run tests**

Run:

```bash
pnpm test scripts/release/workflow-updater.test.mjs
pnpm format:check
````

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add .github/workflows/release-unstable.yml docs/releasing.md scripts/release/workflow-updater.test.mjs
git commit -m "ci: label linux window unstable experiments"
```

---

## Task 5: Publish And QA Variant 1 `baseline-sequenced-logs`

**Files:**

- Create: `docs/qa/linux-native-window-controls-experiments.md`

- [ ] **Step 1: Create the QA result table**

Create `docs/qa/linux-native-window-controls-experiments.md`:

```md
# Linux Native Window Controls Experiment Results

Environment for all rows unless noted:

- Ubuntu Wayland
- Installed package from unstable release
- Test starts without using the titlebar double-click workaround

| Experiment              | Settings first open | About first open | Update first open | Widget first open | Close/reopen regression | Diagnostics bundle | Result  |
| ----------------------- | ------------------- | ---------------- | ----------------- | ----------------- | ----------------------- | ------------------ | ------- |
| baseline-sequenced-logs | Not run             | Not run          | Not run           | Not run           | Not run                 | Not attached       | Pending |
| on-demand-visible       | Not run             | Not run          | Not run           | Not run           | Not run                 | Not attached       | Pending |
| on-demand-hidden        | Not run             | Not run          | Not run           | Not run           | Not run                 | Not attached       | Pending |
| builder-size-only       | Not run             | Not run          | Not run           | Not run           | Not run                 | Not attached       | Pending |
| show-focus-only         | Not run             | Not run          | Not run           | Not run           | Not run                 | Not attached       | Pending |
| unminimize-show-focus   | Not run             | Not run          | Not run           | Not run           | Not run                 | Not attached       | Pending |
```

- [ ] **Step 2: Run local gates before publishing unstable**

Run:

```bash
pnpm lint
pnpm format:check
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: all PASS.

- [ ] **Step 3: Publish unstable diagnostic build**

Use GitHub workflow dispatch:

```bash
gh workflow run release-unstable.yml \
  --ref "$(git branch --show-current)" \
  -f linux_window_experiment=baseline-sequenced-logs
```

Expected: workflow starts and produces an unstable prerelease.

- [ ] **Step 4: Install on Ubuntu Wayland**

On the Ubuntu machine:

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-linux.sh | bash -s -- -i
```

Expected: installed unstable build release notes mention `baseline-sequenced-logs`.

- [ ] **Step 5: Run manual QA**

On Ubuntu Wayland:

```text
1. Open settings.
2. Hover minimize, maximize, close.
3. Click minimize. Reopen settings.
4. Click maximize. Restore. Click close.
5. Repeat for About.
6. Repeat for Update.
7. Repeat for Widget.
8. Close and reopen every surface once.
9. Run: mochi diagnostics --bundle
```

Expected for baseline: native controls likely fail until maximize/restore. Record exact behavior and bundle path.

- [ ] **Step 6: Commit QA result**

Update the `baseline-sequenced-logs` row in `docs/qa/linux-native-window-controls-experiments.md`.

```bash
git add docs/qa/linux-native-window-controls-experiments.md
git commit -m "docs: record linux window baseline experiment"
```

---

## Task 6: Implement Variant 2 `on-demand-visible`

**Files:**

- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/tray/panel.rs`
- Modify: `src-tauri/src/widget/commands.rs`
- Modify: `src-tauri/tauri.conf.json`
- Test: `src/lib/tauri/native-window-controls.test.ts`
- Test: `src-tauri/src/window_policy.rs`

- [ ] **Step 1: Write failing source tests**

In `src/lib/tauri/native-window-controls.test.ts`, add:

```ts
it("does not precreate decorated linux app windows for on-demand-visible", () => {
  const lib = readFileSync(resolve("src-tauri/src/lib.rs"), "utf8");
  const policy = readFileSync(resolve("src-tauri/src/window_policy.rs"), "utf8");

  expect(policy).toContain("OnDemandVisible");
  expect(lib).toContain("should_precreate_decorated_windows_at_startup");
});

it("widget config is not the permanent linux on-demand creation source", () => {
  const commands = readFileSync(resolve("src-tauri/src/widget/commands.rs"), "utf8");

  expect(commands).toContain("build_widget_window");
  expect(commands).toContain("DecoratedWindowInitialVisibility::Visible");
});
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
pnpm test src/lib/tauri/native-window-controls.test.ts
```

Expected: FAIL because the policy is not wired.

- [ ] **Step 3: Add policy decision helpers**

In `src-tauri/src/window_policy.rs`, add:

```rust
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
```

- [ ] **Step 4: Skip startup precreation when policy says on-demand**

In `src-tauri/src/lib.rs`, replace:

```rust
setup_app_windows(app.handle())?;
setup_tray(app.handle())?;
setup_widget(app.handle())?;
```

with:

```rust
if window_policy::should_precreate_decorated_windows_at_startup() {
    setup_app_windows(app.handle())?;
    setup_widget(app.handle())?;
}
setup_tray(app.handle())?;
```

- [ ] **Step 5: Make settings builder use requested visibility**

In `src-tauri/src/tray/panel.rs`, change the settings builder `.visible(false)` call to:

```rust
.visible(matches!(
    crate::window_policy::decorated_window_initial_visibility(),
    crate::window_policy::DecoratedWindowInitialVisibility::Visible
))
```

In `open_app_window()`, only run `set_size()` / `set_min_size()` before first show when policy allows:

```rust
if crate::window_policy::should_mutate_size_before_first_show() {
    let (width, height) = app_window_size_for_path(path.as_str());
    let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }));
    let _ = window.set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize {
        width: if path.starts_with("/about") {
            ABOUT_WINDOW_WIDTH
        } else if path.starts_with("/update") {
            UPDATE_WINDOW_WIDTH
        } else {
            480.0
        },
        height: if path.starts_with("/about") {
            ABOUT_WINDOW_HEIGHT
        } else if path.starts_with("/update") {
            UPDATE_WINDOW_HEIGHT
        } else {
            420.0
        },
    })));
}
```

For `on-demand-visible`, builder defaults to the settings size. Do not attempt route-specific about/update sizing in this variant; record in QA whether controls work before optimizing size.

- [ ] **Step 6: Make widget builder use requested visibility**

In `src-tauri/src/widget/commands.rs`, change:

```rust
.visible(false)
```

to:

```rust
.visible(matches!(
    crate::window_policy::decorated_window_initial_visibility(),
    crate::window_policy::DecoratedWindowInitialVisibility::Visible
))
```

In `show_widget()`, if the policy is `AlreadyVisibleFocus`, skip `show()` and `unminimize()` and only focus:

```rust
match crate::window_policy::first_show_sequence() {
    crate::window_policy::FirstShowSequence::AlreadyVisibleFocus => {
        let focus_result = window.set_focus();
        crate::diagnostics::log_window_action_result(
            WIDGET_LABEL,
            "set_focus",
            focus_result.as_ref().map(|_| ()),
        );
        focus_result.map_err(|error| error.to_string())
    }
    crate::window_policy::FirstShowSequence::ShowUnminimizeFocus => {
        // existing show -> unminimize -> focus path
    }
    crate::window_policy::FirstShowSequence::ShowFocus => {
        // until Task 9 implements this variant, run the existing sequence
        run_show_unminimize_focus(window)
    }
    crate::window_policy::FirstShowSequence::UnminimizeShowFocus => {
        // until Task 10 implements this variant, run the existing sequence
        run_show_unminimize_focus(window)
    }
}
```

Do not leave panic branches in code that can run for a selected experiment. If this task adds the match, fully preserve the existing path for unsupported variants by falling back to `ShowUnminimizeFocus`.

- [ ] **Step 7: Run focused tests**

Run:

```bash
pnpm test src/lib/tauri/native-window-controls.test.ts
cargo test --manifest-path src-tauri/Cargo.toml window_policy
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 8: Publish unstable diagnostic build**

Run local gates from Task 5 Step 2, then:

```bash
gh workflow run release-unstable.yml \
  --ref "$(git branch --show-current)" \
  -f linux_window_experiment=on-demand-visible
```

- [ ] **Step 9: Ubuntu QA**

Install unstable and run the same manual QA script. Expected if Hypothesis A is correct: controls hover/click on first open for settings/about/update/widget and after close/reopen.

- [ ] **Step 10: Commit result**

```bash
git add src-tauri/src/lib.rs src-tauri/src/window_policy.rs src-tauri/src/tray/panel.rs src-tauri/src/widget/commands.rs src/lib/tauri/native-window-controls.test.ts docs/qa/linux-native-window-controls-experiments.md
git commit -m "test: run linux on-demand visible window experiment"
```

---

## Task 7: Implement Variant 3 `on-demand-hidden`

**Condition:** Do this only if `on-demand-visible` fixes native controls.

**Files:**

- Modify: `src-tauri/src/window_policy.rs`
- Modify: `src-tauri/src/tray/panel.rs`
- Modify: `src-tauri/src/widget/commands.rs`
- Modify: `docs/qa/linux-native-window-controls-experiments.md`

- [ ] **Step 1: Confirm policy already represents hidden on-demand**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml on_demand_visible_policy_creates_decorated_windows_on_user_action
```

Expected: PASS. `OnDemandHidden` should already be represented in policy.

- [ ] **Step 2: Publish unstable diagnostic build**

Run:

```bash
gh workflow run release-unstable.yml \
  --ref "$(git branch --show-current)" \
  -f linux_window_experiment=on-demand-hidden
```

- [ ] **Step 3: Ubuntu QA**

Run the manual QA script.

Expected:

- If controls fail, `visible(false)` decorated creation is the root cause.
- If controls pass, startup precreation or frontend boot timing is the root cause.

- [ ] **Step 4: Commit QA result**

```bash
git add docs/qa/linux-native-window-controls-experiments.md
git commit -m "docs: record linux on-demand hidden experiment"
```

---

## Task 8: Implement Variant 4 `builder-size-only`

**Condition:** Do this only if `on-demand-visible` does not fix native controls.

**Files:**

- Modify: `src-tauri/src/tray/panel.rs`
- Modify: `src-tauri/src/widget/commands.rs`
- Modify: `src-tauri/src/window_policy.rs`
- Modify: `docs/qa/linux-native-window-controls-experiments.md`

- [ ] **Step 1: Write failing Rust test for no pre-show size mutation**

In `src-tauri/src/window_policy.rs`, add:

```rust
#[test]
fn builder_size_only_disables_pre_show_size_mutation() {
    let policy = LinuxWindowPolicy::for_experiment(LinuxWindowExperiment::BuilderSizeOnly);
    assert!(!policy.mutate_size_before_first_show);
    assert_eq!(policy.creation_mode, DecoratedWindowCreationMode::OnDemand);
}
```

- [ ] **Step 2: Run test**

```bash
cargo test --manifest-path src-tauri/Cargo.toml builder_size_only_disables_pre_show_size_mutation
```

Expected: PASS if Task 1 was implemented correctly.

- [ ] **Step 3: Ensure app window open respects policy**

In `open_app_window()`, verify the only calls to `window.set_size()` and `window.set_min_size()` are inside:

```rust
if crate::window_policy::should_mutate_size_before_first_show() {
    // set_size / set_min_size
}
```

In `show_widget()`, verify no height sync is called before first show. Frontend `set_widget_height()` calls may still occur after webview boot; diagnostics must show when they occur.

- [ ] **Step 4: Publish unstable diagnostic build**

```bash
gh workflow run release-unstable.yml \
  --ref "$(git branch --show-current)" \
  -f linux_window_experiment=builder-size-only
```

- [ ] **Step 5: Ubuntu QA**

Run the manual QA script.

Expected if size mutation is root cause: controls work on first open and after close/reopen.

- [ ] **Step 6: Commit QA result**

```bash
git add src-tauri/src/window_policy.rs src-tauri/src/tray/panel.rs src-tauri/src/widget/commands.rs docs/qa/linux-native-window-controls-experiments.md
git commit -m "test: run linux builder size window experiment"
```

---

## Task 9: Implement Variant 5 `show-focus-only`

**Condition:** Do this only if `builder-size-only` does not fix native controls.

**Files:**

- Modify: `src-tauri/src/tray/panel.rs`
- Modify: `src-tauri/src/widget/commands.rs`
- Modify: `docs/qa/linux-native-window-controls-experiments.md`

- [ ] **Step 1: Add a shared first-show helper**

In `src-tauri/src/tray/panel.rs`, add:

```rust
fn show_decorated_window(window: &WebviewWindow, label: &str) -> Result<(), String> {
    match crate::window_policy::first_show_sequence() {
        crate::window_policy::FirstShowSequence::AlreadyVisibleFocus => {
            let focus_result = window.set_focus();
            crate::diagnostics::log_window_action_result(label, "set_focus", focus_result.as_ref().map(|_| ()));
            focus_result.map_err(|error| error.to_string())
        }
        crate::window_policy::FirstShowSequence::ShowFocus => {
            let show_result = window.show();
            crate::diagnostics::log_window_action_result(label, "show", show_result.as_ref().map(|_| ()));
            show_result.map_err(|error| error.to_string())?;
            let focus_result = window.set_focus();
            crate::diagnostics::log_window_action_result(label, "set_focus", focus_result.as_ref().map(|_| ()));
            focus_result.map_err(|error| error.to_string())
        }
        crate::window_policy::FirstShowSequence::UnminimizeShowFocus => {
            let unminimize_result = window.unminimize();
            crate::diagnostics::log_window_action_result(label, "unminimize", unminimize_result.as_ref().map(|_| ()));
            unminimize_result.map_err(|error| error.to_string())?;
            let show_result = window.show();
            crate::diagnostics::log_window_action_result(label, "show", show_result.as_ref().map(|_| ()));
            show_result.map_err(|error| error.to_string())?;
            let focus_result = window.set_focus();
            crate::diagnostics::log_window_action_result(label, "set_focus", focus_result.as_ref().map(|_| ()));
            focus_result.map_err(|error| error.to_string())
        }
        crate::window_policy::FirstShowSequence::ShowUnminimizeFocus => {
            let show_result = window.show();
            crate::diagnostics::log_window_action_result(label, "show", show_result.as_ref().map(|_| ()));
            show_result.map_err(|error| error.to_string())?;
            let unminimize_result = window.unminimize();
            crate::diagnostics::log_window_action_result(label, "unminimize", unminimize_result.as_ref().map(|_| ()));
            unminimize_result.map_err(|error| error.to_string())?;
            let focus_result = window.set_focus();
            crate::diagnostics::log_window_action_result(label, "set_focus", focus_result.as_ref().map(|_| ()));
            focus_result.map_err(|error| error.to_string())
        }
    }
}
```

Use the helper in `open_app_window()` and duplicate or move it to a shared module for widget. If duplicated temporarily, keep both copies byte-for-byte similar and remove duplication in the final proven fix task.

- [ ] **Step 2: Publish unstable diagnostic build**

```bash
gh workflow run release-unstable.yml \
  --ref "$(git branch --show-current)" \
  -f linux_window_experiment=show-focus-only
```

- [ ] **Step 3: Ubuntu QA**

Run the manual QA script.

Expected if show sequencing is root cause: controls work only in this variant or later show-order variants.

- [ ] **Step 4: Commit QA result**

```bash
git add src-tauri/src/tray/panel.rs src-tauri/src/widget/commands.rs docs/qa/linux-native-window-controls-experiments.md
git commit -m "test: run linux show focus window experiment"
```

---

## Task 10: Implement Variant 6 `unminimize-show-focus`

**Condition:** Do this only if `show-focus-only` does not fix native controls.

**Files:**

- Modify: `src-tauri/src/window_policy.rs`
- Modify: `src-tauri/src/tray/panel.rs`
- Modify: `src-tauri/src/widget/commands.rs`
- Modify: `docs/qa/linux-native-window-controls-experiments.md`

- [ ] **Step 1: Verify helper supports the sequence**

Run:

```bash
rg -n "UnminimizeShowFocus|unminimize.*show.*set_focus|unminimize-show-focus" src-tauri/src
```

Expected: `FirstShowSequence::UnminimizeShowFocus` is implemented in the show helper and policy parser.

- [ ] **Step 2: Publish unstable diagnostic build**

```bash
gh workflow run release-unstable.yml \
  --ref "$(git branch --show-current)" \
  -f linux_window_experiment=unminimize-show-focus
```

- [ ] **Step 3: Ubuntu QA**

Run the manual QA script.

- [ ] **Step 4: Commit QA result**

```bash
git add src-tauri/src/window_policy.rs src-tauri/src/tray/panel.rs src-tauri/src/widget/commands.rs docs/qa/linux-native-window-controls-experiments.md
git commit -m "test: run linux unminimize show focus experiment"
```

---

## Task 11: Promote The Proven Fix And Remove Experiment Scaffolding

**Condition:** Do this only after an unstable installed build fixes the issue on Ubuntu Wayland.

**Files:**

- Modify: `src-tauri/src/window_policy.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/tray/panel.rs`
- Modify: `src-tauri/src/widget/commands.rs`
- Modify: `.github/workflows/release-unstable.yml`
- Modify: `docs/qa/linux-native-window-controls-experiments.md`

- [ ] **Step 1: Write final policy tests**

In `src-tauri/src/window_policy.rs`, replace experiment-specific final tests with:

```rust
#[test]
fn linux_uses_the_proven_decorated_window_policy() {
    let policy = proven_linux_decorated_window_policy();
    assert_eq!(policy.creation_mode, DecoratedWindowCreationMode::OnDemand);
    assert_eq!(policy.initial_visibility, DecoratedWindowInitialVisibility::Visible);
    assert!(!policy.mutate_size_before_first_show);
}
```

Adjust expected values to match the actually proven variant.

- [ ] **Step 2: Remove unproven runtime branches**

Remove `MOCHI_LINUX_WINDOW_EXPERIMENT` from normal application behavior. Keep diagnostics that report:

```text
decorated_window_policy=<proven-policy-name>
```

Do not keep inactive branches that were only useful for experiments.

- [ ] **Step 3: Simplify release workflow**

Remove the `linux_window_experiment` input from `.github/workflows/release-unstable.yml` after the proven fix is promoted.

Keep release notes simple:

```markdown
- Fixed Linux native window controls by using the proven decorated window lifecycle policy.
```

- [ ] **Step 4: Run full verification**

Run:

```bash
pnpm lint
pnpm format:check
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: all PASS.

- [ ] **Step 5: Publish final unstable**

```bash
gh workflow run release-unstable.yml --ref "$(git branch --show-current)"
```

- [ ] **Step 6: Final Ubuntu QA**

Install final unstable and run the full manual QA. Acceptance requires:

- settings controls hover/click on first open
- about controls hover/click on first open
- update controls hover/click on first open
- widget controls hover/click on first open
- close/reopen does not regress
- widget background matches settings
- widget scrolling still works
- no maximize/restore workaround is needed

- [ ] **Step 7: Commit final fix**

```bash
git add src-tauri/src/window_policy.rs src-tauri/src/lib.rs src-tauri/src/tray/panel.rs src-tauri/src/widget/commands.rs .github/workflows/release-unstable.yml docs/qa/linux-native-window-controls-experiments.md
git commit -m "fix: use proven linux decorated window lifecycle"
```

---

## Final Verification Checklist

Before claiming the work is complete:

- [ ] Latest unstable installed package was tested on Ubuntu Wayland.
- [ ] `mochi diagnostics --bundle` attached or recorded for the winning variant.
- [ ] QA table says `Pass` for the winning variant.
- [ ] Widget background visually matches settings.
- [ ] The final code does not keep unproven experiment branches.
- [ ] Full frontend and Rust gates pass.
