# Linux Window Controls On-Demand Promotion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote the proven Linux `on-demand-visible` decorated-window behavior to the default production behavior and remove temporary experiment scaffolding.

**Architecture:** Replace compile-time experiment selection with a permanent platform window policy. Linux decorated settings/widget windows are created on demand and visible; macOS/Windows keep startup-hidden behavior. Release workflows no longer accept or publish experiment labels.

**Tech Stack:** Tauri v2 Rust backend, React 19 + Vite 8 frontend, Vitest source tests, Rust unit tests, GitHub Actions release workflows.

---

## Proven Inputs

- Broken behavior: hidden-precreated decorated Linux windows logged `creation=startup-precreate initial_visibility=hidden` and native controls did not work.
- Proven fix: `unstable-20260606.233705` with a truly compiled `on-demand-visible` binary made settings and widget controls work.
- Release trap to avoid: release notes can say `on-demand-visible` while the binary remains baseline if Cargo cache is not invalidated. This plan removes the experiment env entirely, so no future build should depend on `MOCHI_LINUX_WINDOW_EXPERIMENT`.

---

## File Structure

- Modify: `src-tauri/src/window_policy.rs`
  - Replace experiment policy with permanent platform policy.
- Modify: `src-tauri/src/lib.rs`
  - Keep startup precreation only when platform policy says to precreate.
- Modify: `src-tauri/src/tray/panel.rs`
  - Replace experiment-based lifecycle strings and match arms with permanent policy.
- Modify: `src-tauri/src/widget/commands.rs`
  - Replace experiment-based lifecycle strings and match arms with permanent policy.
- Modify: `src-tauri/src/diagnostics/state.rs`
  - Rename lifecycle detail field from `experiment=` to `policy=`.
- Modify: `src-tauri/src/diagnostics/report.rs`
  - Ensure diagnostics bundle still includes window lifecycle lines.
- Modify: `src-tauri/src/frontend.rs`
  - Keep tests proving widget is Rust-created and uses SPA shell.
- Modify: `src-tauri/tauri.conf.json`
  - Keep `app.windows` empty; do not re-add `widget`.
- Modify: `src-tauri/build.rs`
  - Remove `MOCHI_LINUX_WINDOW_EXPERIMENT` Cargo env wiring after experiment code is gone.
- Modify: `.github/workflows/release-unstable.yml`
  - Remove `linux_window_experiment` workflow input/env and release-note label.
- Modify: `scripts/release/workflow-updater.test.mjs`
  - Replace experiment-wiring test with a test proving the experiment env is absent.
- Modify: `src/lib/tauri/native-window-controls.test.ts`
  - Add source assertions for permanent Linux on-demand policy and no experiment env.
- Modify: `src/components/widget/widget-window.test.ts`
  - Keep source assertion that widget is not statically configured.
- Modify: `docs/releasing.md`
  - Remove Linux diagnostic experiment instructions or archive them as historical notes.
- Modify: `docs/qa/linux-native-window-controls-experiments.md`
  - Record the final successful `on-demand-visible` result and mark the experiment complete.

---

## Task 1: Replace Experiment Policy With Permanent Platform Policy

**Files:**

- Modify: `src-tauri/src/window_policy.rs`
- Test: `src-tauri/src/window_policy.rs`

- [ ] **Step 1: Write failing Rust tests**

Replace the `#[cfg(test)] mod tests` section in `src-tauri/src/window_policy.rs` with:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_policy_uses_on_demand_visible_decorated_windows() {
        let policy = DecoratedWindowPolicy::for_target_os("linux");

        assert_eq!(policy.name, "linux-on-demand-visible");
        assert_eq!(policy.creation_mode, DecoratedWindowCreationMode::OnDemand);
        assert_eq!(
            policy.initial_visibility,
            DecoratedWindowInitialVisibility::Visible
        );
        assert_eq!(policy.first_show_sequence, FirstShowSequence::AlreadyVisibleFocus);
        assert!(!policy.mutate_size_before_first_show);
    }

    #[test]
    fn macos_and_windows_keep_startup_hidden_policy() {
        for os in ["macos", "windows"] {
            let policy = DecoratedWindowPolicy::for_target_os(os);

            assert_eq!(policy.name, "startup-hidden");
            assert_eq!(
                policy.creation_mode,
                DecoratedWindowCreationMode::StartupPrecreate
            );
            assert_eq!(
                policy.initial_visibility,
                DecoratedWindowInitialVisibility::Hidden
            );
            assert_eq!(
                policy.first_show_sequence,
                FirstShowSequence::ShowUnminimizeFocus
            );
            assert!(policy.mutate_size_before_first_show);
        }
    }

    #[test]
    fn unknown_targets_use_startup_hidden_policy() {
        let policy = DecoratedWindowPolicy::for_target_os("freebsd");

        assert_eq!(policy.name, "startup-hidden");
        assert_eq!(
            policy.creation_mode,
            DecoratedWindowCreationMode::StartupPrecreate
        );
    }
}
```

- [ ] **Step 2: Run the tests and verify they fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml window_policy
```

Expected: FAIL because `DecoratedWindowPolicy` does not exist and old experiment types still exist.

- [ ] **Step 3: Replace `src-tauri/src/window_policy.rs`**

Replace the entire file with:

```rust
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecoratedWindowPolicy {
    pub name: &'static str,
    pub creation_mode: DecoratedWindowCreationMode,
    pub initial_visibility: DecoratedWindowInitialVisibility,
    pub first_show_sequence: FirstShowSequence,
    pub mutate_size_before_first_show: bool,
}

impl DecoratedWindowPolicy {
    pub const fn for_target_os(target_os: &str) -> Self {
        match target_os {
            "linux" => Self {
                name: "linux-on-demand-visible",
                creation_mode: DecoratedWindowCreationMode::OnDemand,
                initial_visibility: DecoratedWindowInitialVisibility::Visible,
                first_show_sequence: FirstShowSequence::AlreadyVisibleFocus,
                mutate_size_before_first_show: false,
            },
            _ => Self {
                name: "startup-hidden",
                creation_mode: DecoratedWindowCreationMode::StartupPrecreate,
                initial_visibility: DecoratedWindowInitialVisibility::Hidden,
                first_show_sequence: FirstShowSequence::ShowUnminimizeFocus,
                mutate_size_before_first_show: true,
            },
        }
    }

    pub const fn creation_label(self) -> &'static str {
        match self.creation_mode {
            DecoratedWindowCreationMode::StartupPrecreate => "startup-precreate",
            DecoratedWindowCreationMode::OnDemand => "on-demand",
        }
    }

    pub const fn initial_visibility_label(self) -> &'static str {
        match self.initial_visibility {
            DecoratedWindowInitialVisibility::Hidden => "hidden",
            DecoratedWindowInitialVisibility::Visible => "visible",
        }
    }
}

pub fn active_decorated_window_policy() -> DecoratedWindowPolicy {
    DecoratedWindowPolicy::for_target_os(std::env::consts::OS)
}

pub fn should_precreate_decorated_windows_at_startup() -> bool {
    active_decorated_window_policy().creation_mode == DecoratedWindowCreationMode::StartupPrecreate
}

pub fn decorated_window_initial_visibility() -> DecoratedWindowInitialVisibility {
    active_decorated_window_policy().initial_visibility
}

pub fn should_mutate_size_before_first_show() -> bool {
    active_decorated_window_policy().mutate_size_before_first_show
}

pub fn first_show_sequence() -> FirstShowSequence {
    active_decorated_window_policy().first_show_sequence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_policy_uses_on_demand_visible_decorated_windows() {
        let policy = DecoratedWindowPolicy::for_target_os("linux");

        assert_eq!(policy.name, "linux-on-demand-visible");
        assert_eq!(policy.creation_mode, DecoratedWindowCreationMode::OnDemand);
        assert_eq!(
            policy.initial_visibility,
            DecoratedWindowInitialVisibility::Visible
        );
        assert_eq!(
            policy.first_show_sequence,
            FirstShowSequence::AlreadyVisibleFocus
        );
        assert!(!policy.mutate_size_before_first_show);
    }

    #[test]
    fn macos_and_windows_keep_startup_hidden_policy() {
        for os in ["macos", "windows"] {
            let policy = DecoratedWindowPolicy::for_target_os(os);

            assert_eq!(policy.name, "startup-hidden");
            assert_eq!(
                policy.creation_mode,
                DecoratedWindowCreationMode::StartupPrecreate
            );
            assert_eq!(
                policy.initial_visibility,
                DecoratedWindowInitialVisibility::Hidden
            );
            assert_eq!(
                policy.first_show_sequence,
                FirstShowSequence::ShowUnminimizeFocus
            );
            assert!(policy.mutate_size_before_first_show);
        }
    }

    #[test]
    fn unknown_targets_use_startup_hidden_policy() {
        let policy = DecoratedWindowPolicy::for_target_os("freebsd");

        assert_eq!(policy.name, "startup-hidden");
        assert_eq!(
            policy.creation_mode,
            DecoratedWindowCreationMode::StartupPrecreate
        );
    }
}
```

- [ ] **Step 4: Run the tests and verify they pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml window_policy
```

Expected: PASS with 3 `window_policy` tests.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/window_policy.rs
git commit -m "fix(linux): promote on-demand window policy"
```

---

## Task 2: Make Settings Diagnostics And Show Flow Use Permanent Policy

**Files:**

- Modify: `src-tauri/src/tray/panel.rs`
- Test: `src/lib/tauri/native-window-controls.test.ts`

- [ ] **Step 1: Add failing source assertions**

In `src/lib/tauri/native-window-controls.test.ts`, add:

```ts
it("uses permanent policy labels for linux app window lifecycle", () => {
  const source = readFileSync(resolve("src-tauri/src/tray/panel.rs"), "utf8");

  expect(source).toContain("active_decorated_window_policy()");
  expect(source).toContain("policy.creation_label()");
  expect(source).toContain("policy.initial_visibility_label()");
  expect(source).not.toContain('startup-precreate", "hidden"');
});
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
pnpm test src/lib/tauri/native-window-controls.test.ts
```

Expected: FAIL because `panel.rs` still hardcodes lifecycle strings such as `"startup-precreate", "hidden"`.

- [ ] **Step 3: Replace hardcoded app-window lifecycle labels**

In `src-tauri/src/tray/panel.rs`, inside `open_app_window()`, add after `let window = ensure_settings_window(&app)?;`:

```rust
    let policy = crate::window_policy::active_decorated_window_policy();
    let creation = policy.creation_label();
    let initial_visibility = policy.initial_visibility_label();
```

Replace:

```rust
    record_app_window_lifecycle(&window, "created", "startup-precreate", "hidden");
```

with:

```rust
    record_app_window_lifecycle(&window, "created", creation, initial_visibility);
```

Replace each remaining hardcoded lifecycle call in `open_app_window()` exactly as follows.

Replace:

```rust
record_app_window_lifecycle(&window, "before-set-size", "startup-precreate", "hidden");
```

with:

```rust
record_app_window_lifecycle(&window, "before-set-size", creation, initial_visibility);
```

Replace:

```rust
record_app_window_lifecycle(&window, "after-set-size", "startup-precreate", "hidden");
```

with:

```rust
record_app_window_lifecycle(&window, "after-set-size", creation, initial_visibility);
```

Replace:

```rust
record_app_window_lifecycle(&window, "after-show", "startup-precreate", "hidden");
```

with:

```rust
record_app_window_lifecycle(&window, "after-show", creation, initial_visibility);
```

Replace:

```rust
record_app_window_lifecycle(
    &window,
    "after-unminimize",
    "startup-precreate",
    "hidden",
);
```

with:

```rust
record_app_window_lifecycle(&window, "after-unminimize", creation, initial_visibility);
```

Replace:

```rust
record_app_window_lifecycle(&window, "after-focus", "startup-precreate", "hidden");
```

with:

```rust
record_app_window_lifecycle(&window, "after-focus", creation, initial_visibility);
```

- [ ] **Step 4: Remove experiment-only fallback match arm**

In `open_app_window()`, ensure the match has only these arms:

```rust
    match crate::window_policy::first_show_sequence() {
        crate::window_policy::FirstShowSequence::AlreadyVisibleFocus => {
            let focus_result = window.set_focus();
            crate::diagnostics::log_window_action_result(
                SETTINGS_WINDOW_LABEL,
                "set_focus",
                focus_result.as_ref().map(|_| ()),
            );
            record_app_window_lifecycle(&window, "after-focus", creation, initial_visibility);
            focus_result.map_err(|error| error.to_string())?;
        }
        crate::window_policy::FirstShowSequence::ShowUnminimizeFocus => {
            if window.is_visible().unwrap_or(false) {
                let focus_result = window.set_focus();
                crate::diagnostics::log_window_action_result(
                    SETTINGS_WINDOW_LABEL,
                    "set_focus",
                    focus_result.as_ref().map(|_| ()),
                );
                record_app_window_lifecycle(&window, "after-focus", creation, initial_visibility);
                focus_result.map_err(|error| error.to_string())?;
            } else {
                let show_result = window.show();
                crate::diagnostics::log_window_action_result(
                    SETTINGS_WINDOW_LABEL,
                    "show",
                    show_result.as_ref().map(|_| ()),
                );
                show_result.map_err(|error| error.to_string())?;
                record_app_window_lifecycle(&window, "after-show", creation, initial_visibility);
                record_app_window_controls(&window, "rust-builder");

                let unminimize_result = window.unminimize();
                crate::diagnostics::log_window_action_result(
                    SETTINGS_WINDOW_LABEL,
                    "unminimize",
                    unminimize_result.as_ref().map(|_| ()),
                );
                unminimize_result.map_err(|error| error.to_string())?;
                record_app_window_lifecycle(&window, "after-unminimize", creation, initial_visibility);
                record_app_window_controls(&window, "rust-builder");

                let focus_result = window.set_focus();
                crate::diagnostics::log_window_action_result(
                    SETTINGS_WINDOW_LABEL,
                    "set_focus",
                    focus_result.as_ref().map(|_| ()),
                );
                record_app_window_lifecycle(&window, "after-focus", creation, initial_visibility);
                focus_result.map_err(|error| error.to_string())?;
            }
        }
    }
```

- [ ] **Step 5: Run focused tests**

Run:

```bash
pnpm test src/lib/tauri/native-window-controls.test.ts
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/tray/panel.rs src/lib/tauri/native-window-controls.test.ts
git commit -m "fix(linux): use permanent app window lifecycle policy"
```

---

## Task 3: Make Widget Diagnostics And Show Flow Use Permanent Policy

**Files:**

- Modify: `src-tauri/src/widget/commands.rs`
- Test: `src/lib/tauri/native-window-controls.test.ts`
- Test: `src/components/widget/widget-window.test.ts`

- [ ] **Step 1: Add failing source assertions**

In `src/lib/tauri/native-window-controls.test.ts`, add:

```ts
it("uses permanent policy labels for widget lifecycle", () => {
  const source = readFileSync(resolve("src-tauri/src/widget/commands.rs"), "utf8");

  expect(source).toContain("active_decorated_window_policy()");
  expect(source).toContain("policy.creation_label()");
  expect(source).toContain("policy.initial_visibility_label()");
  expect(source).not.toContain("tauri-config");
});
```

In `src/components/widget/widget-window.test.ts`, keep or add:

```ts
it("keeps the widget out of static tauri config so linux can create it on demand", () => {
  const config = JSON.parse(readFileSync(resolve("src-tauri/tauri.conf.json"), "utf8"));
  const widget = config.app.windows.find((window: { label: string }) => window.label === "widget");
  expect(widget).toBeUndefined();
});
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
pnpm test src/lib/tauri/native-window-controls.test.ts src/components/widget/widget-window.test.ts
```

Expected: FAIL if widget code still has experiment labels or `tauri-config` as the permanent creation source.

- [ ] **Step 3: Update `setup_widget()` lifecycle labels**

In `src-tauri/src/widget/commands.rs`, inside `setup_widget()`, add before `record_widget_window_lifecycle`:

```rust
    let policy = crate::window_policy::active_decorated_window_policy();
```

Replace:

```rust
    record_widget_window_lifecycle(&window, "created", "startup-precreate", "hidden");
```

with:

```rust
    record_widget_window_lifecycle(
        &window,
        "created",
        policy.creation_label(),
        policy.initial_visibility_label(),
    );
```

- [ ] **Step 4: Update `show_widget()` policy handling**

In `show_widget()`, add after `let window = ensure_widget_window(&app)?;`:

```rust
    let policy = crate::window_policy::active_decorated_window_policy();
    let creation = policy.creation_label();
    let initial_visibility = policy.initial_visibility_label();
```

Replace:

```rust
record_widget_window_controls(&app, &window, "tauri-config");
```

with:

```rust
record_widget_window_controls(&app, &window, creation);
```

Replace the `match` with only:

```rust
    match crate::window_policy::first_show_sequence() {
        crate::window_policy::FirstShowSequence::AlreadyVisibleFocus => {
            let focus_result = window.set_focus();
            crate::diagnostics::log_window_action_result(
                WIDGET_LABEL,
                "set_focus",
                focus_result.as_ref().map(|_| ()),
            );
            record_widget_window_lifecycle(&window, "after-focus", creation, initial_visibility);
            focus_result.map_err(|error| error.to_string())
        }
        crate::window_policy::FirstShowSequence::ShowUnminimizeFocus => {
            let show_result = window.show();
            crate::diagnostics::log_window_action_result(
                WIDGET_LABEL,
                "show",
                show_result.as_ref().map(|_| ()),
            );
            show_result.map_err(|error| error.to_string())?;
            record_widget_window_lifecycle(&window, "after-show", creation, initial_visibility);

            let unminimize_result = window.unminimize();
            crate::diagnostics::log_window_action_result(
                WIDGET_LABEL,
                "unminimize",
                unminimize_result.as_ref().map(|_| ()),
            );
            unminimize_result.map_err(|error| error.to_string())?;
            record_widget_window_lifecycle(&window, "after-unminimize", creation, initial_visibility);
            record_widget_window_controls(&app, &window, creation);

            let focus_result = window.set_focus();
            crate::diagnostics::log_window_action_result(
                WIDGET_LABEL,
                "set_focus",
                focus_result.as_ref().map(|_| ()),
            );
            record_widget_window_lifecycle(&window, "after-focus", creation, initial_visibility);
            focus_result.map_err(|error| error.to_string())
        }
    }
```

- [ ] **Step 5: Update `ensure_widget_window()` lifecycle labels**

Replace:

```rust
record_widget_window_lifecycle(&window, "created", "on-demand", "visible");
```

with:

```rust
let policy = crate::window_policy::active_decorated_window_policy();
record_widget_window_lifecycle(
    &window,
    "created",
    policy.creation_label(),
    policy.initial_visibility_label(),
);
```

- [ ] **Step 6: Run focused tests**

Run:

```bash
pnpm test src/lib/tauri/native-window-controls.test.ts src/components/widget/widget-window.test.ts
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/widget/commands.rs src/lib/tauri/native-window-controls.test.ts src/components/widget/widget-window.test.ts
git commit -m "fix(linux): use permanent widget window policy"
```

---

## Task 4: Rename Lifecycle Diagnostics From Experiment To Policy

**Files:**

- Modify: `src-tauri/src/diagnostics/state.rs`
- Test: `src-tauri/src/diagnostics/state.rs`

- [ ] **Step 1: Update failing Rust test**

In `src-tauri/src/diagnostics/state.rs`, update the lifecycle detail test to:

```rust
#[test]
fn window_lifecycle_detail_includes_policy_and_sequence() {
    let detail = window_lifecycle_detail(
        "settings",
        "show",
        "linux-on-demand-visible",
        "on-demand",
        "visible",
        Some((520.0, 560.0)),
        Some((520.0, 560.0)),
    );

    assert!(detail.contains("label=settings"));
    assert!(detail.contains("phase=show"));
    assert!(detail.contains("policy=linux-on-demand-visible"));
    assert!(detail.contains("creation=on-demand"));
    assert!(detail.contains("initial_visibility=visible"));
    assert!(detail.contains("outer=520x560"));
    assert!(detail.contains("inner=520x560"));
    assert!(!detail.contains("experiment="));
}
```

- [ ] **Step 2: Run test and verify failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml window_lifecycle_detail
```

Expected: FAIL because diagnostics still write `experiment=`.

- [ ] **Step 3: Rename function argument and detail field**

In `window_lifecycle_detail()`, rename the third argument from `experiment` to `policy` and change:

```rust
"label={label} phase={phase} experiment={experiment} creation={creation} initial_visibility={initial_visibility} outer={} inner={}"
```

to:

```rust
"label={label} phase={phase} policy={policy} creation={creation} initial_visibility={initial_visibility} outer={} inner={}"
```

Update `record_window_lifecycle()` parameter name from `experiment` to `policy` and pass it through.

- [ ] **Step 4: Update call sites**

In `src-tauri/src/tray/panel.rs` and `src-tauri/src/widget/commands.rs`, replace:

```rust
let experiment = crate::window_policy::active_linux_window_experiment().name();
```

with:

```rust
let policy = crate::window_policy::active_decorated_window_policy().name;
```

In both files, update the call to `state.record_window_lifecycle` so the third argument is `policy`.

- [ ] **Step 5: Run focused tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml window_lifecycle_detail
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/diagnostics/state.rs src-tauri/src/tray/panel.rs src-tauri/src/widget/commands.rs
git commit -m "chore(diagnostics): report window policy instead of experiment"
```

---

## Task 5: Remove Release Experiment Wiring

**Files:**

- Modify: `.github/workflows/release-unstable.yml`
- Modify: `src-tauri/build.rs`
- Modify: `scripts/release/workflow-updater.test.mjs`

- [ ] **Step 1: Write failing release wiring test**

Replace the `linux window experiment build wiring` test in `scripts/release/workflow-updater.test.mjs` with:

```js
describe("linux window experiment cleanup", () => {
  it("does not publish temporary linux window experiment controls", () => {
    const workflow = readFileSync(".github/workflows/release-unstable.yml", "utf8");
    const buildScript = readFileSync("src-tauri/build.rs", "utf8");
    const policy = readFileSync("src-tauri/src/window_policy.rs", "utf8");

    expect(workflow).not.toContain("linux_window_experiment");
    expect(workflow).not.toContain("MOCHI_LINUX_WINDOW_EXPERIMENT");
    expect(workflow).not.toContain("Linux window experiment:");
    expect(buildScript).not.toContain("MOCHI_LINUX_WINDOW_EXPERIMENT");
    expect(policy).not.toContain("MOCHI_LINUX_WINDOW_EXPERIMENT");
    expect(policy).not.toContain("LinuxWindowExperiment");
  });
});
```

- [ ] **Step 2: Run test and verify failure**

Run:

```bash
pnpm test scripts/release/workflow-updater.test.mjs
```

Expected: FAIL while workflow/build script still mention `MOCHI_LINUX_WINDOW_EXPERIMENT`.

- [ ] **Step 3: Remove workflow dispatch input**

In `.github/workflows/release-unstable.yml`, replace:

```yaml
workflow_dispatch:
  inputs:
    linux_window_experiment:
```

and its full nested block with:

```yaml
workflow_dispatch:
```

Remove this env entry from the Tauri build step:

```yaml
MOCHI_LINUX_WINDOW_EXPERIMENT: ${{ github.event.inputs.linux_window_experiment || 'baseline-sequenced-logs' }}
```

Remove this line from both release body locations:

```text
- Linux window experiment: `${{ github.event.inputs.linux_window_experiment || 'baseline-sequenced-logs' }}`
```

Remove this JavaScript line from the `release-notes` job:

```js
const experiment =
  "${{ github.event.inputs.linux_window_experiment || 'baseline-sequenced-logs' }}";
```

Remove this body array entry:

```js
`- Linux window experiment: \`${experiment}\``,
```

- [ ] **Step 4: Remove build script env forwarding**

Replace `src-tauri/build.rs` with:

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 5: Run tests**

Run:

```bash
pnpm test scripts/release/workflow-updater.test.mjs
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add .github/workflows/release-unstable.yml src-tauri/build.rs scripts/release/workflow-updater.test.mjs
git commit -m "chore(release): remove linux window experiment wiring"
```

---

## Task 6: Update Documentation And QA Results

**Files:**

- Modify: `docs/releasing.md`
- Modify: `docs/qa/linux-native-window-controls-experiments.md`
- Test: `scripts/release/workflow-updater.test.mjs`

- [ ] **Step 1: Update `docs/releasing.md`**

Remove the section titled:

```markdown
## Linux Window-Control Diagnostic Unstable Builds
```

Replace it with:

```markdown
## Linux Window Controls

Linux decorated app windows are created on demand and visible. This avoids Ubuntu Wayland native titlebar hit-region failures caused by hidden precreation of decorated windows.

Do not add `MOCHI_LINUX_WINDOW_EXPERIMENT` back to release workflows. The proven behavior is now the default Linux behavior.
```

- [ ] **Step 2: Update QA result table**

In `docs/qa/linux-native-window-controls-experiments.md`, set:

```markdown
| on-demand-visible | Pass | Pass | Pass | Pass | Pass | unstable-20260606.233705 | Proven; promoted |
```

Add a short note below the table:

```markdown
Final result: `on-demand-visible` fixed native controls for settings and widget on Ubuntu Wayland. The behavior is promoted to the permanent Linux policy. Remaining variants were not needed.
```

- [ ] **Step 3: Run docs-adjacent tests**

Run:

```bash
pnpm test scripts/release/workflow-updater.test.mjs
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add docs/releasing.md docs/qa/linux-native-window-controls-experiments.md
git commit -m "docs(linux): record promoted window controls fix"
```

---

## Task 7: Final Verification

**Files:**

- All files changed above.

- [ ] **Step 1: Run full validation**

Run:

```bash
pnpm lint
pnpm format:check
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: all commands PASS.

- [ ] **Step 2: Check experiment strings are gone**

Run:

```bash
rg -n "MOCHI_LINUX_WINDOW_EXPERIMENT|LinuxWindowExperiment|baseline-sequenced-logs|on-demand-hidden|builder-size-only|show-focus-only|unminimize-show-focus|Linux window experiment" src-tauri .github scripts src docs/releasing.md
```

Expected: no matches. If `docs/superpowers/` still contains historical specs/plans, do not include that directory in this check.

- [ ] **Step 3: Manual Ubuntu Wayland smoke test after release**

After this work is merged and an unstable or stable package is published, install it on Ubuntu Wayland and verify:

```bash
mochi diagnostics --bundle
```

Expected diagnostics for first widget open include:

```text
label=widget phase=created policy=linux-on-demand-visible creation=on-demand initial_visibility=visible
```

Expected diagnostics for first settings open include:

```text
label=settings phase=created policy=linux-on-demand-visible creation=on-demand initial_visibility=visible
```

Manual expected behavior:

- Settings native controls hover and click immediately.
- Widget native controls hover and click immediately.
- Closing and reopening each window keeps controls working.

- [ ] **Step 4: Commit any final fixes**

If validation required edits, commit them:

```bash
git add <changed-files>
git commit -m "fix(linux): finalize native window controls promotion"
```

If no edits were needed, do not create an empty commit.

---

## Notes For Executor

- Do not continue testing `on-demand-hidden`, `builder-size-only`, `show-focus-only`, or `unminimize-show-focus`; `on-demand-visible` has already been proven.
- Do not reintroduce the `widget` window in `src-tauri/tauri.conf.json`.
- Do not reintroduce runtime `set_decorations(true)` or `set_resizable(true)` for decorated windows.
- Do not rely on GitHub release notes to prove the compiled behavior. Diagnostics from the installed binary are the source of truth.
