# Selected Tab Persistence via Settings — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate the per-window localStorage tab preference by moving `selected_tab` into the shared `MochiSettings` JSON. The widget reads it synchronously via `initialization_script` on first open, and via a `"set-tab"` event on re-show.

**Architecture:** Add `selected_tab: Option<String>` to the Rust `MochiSettings` struct. On widget creation, inject it as `window.__MOCHI_SELECTED_TAB__` via Tauri's `initialization_script`. On widget re-show, emit `"set-tab"` before `show()`. On tab change, persist to settings via existing `save_settings` command.

**Tech Stack:** Rust (Tauri v2, serde), TypeScript (Zustand, TanStack Query)

---

## 2026-06-20 Security and Concurrency Repair

PR #88 implemented the original plan. The follow-up below supersedes the unsafe
string interpolation in Task 2 and the whole-settings tab write in Task 5.

### Task A: Serialize and normalize the selected tab

**Files:**

- Modify: `src-tauri/src/widget/commands.rs`
- Modify: `src-tauri/src/settings/mod.rs`
- Modify: `src-tauri/src/settings/commands.rs`

- [ ] **Step 1: Add failing Rust regression tests**

Test that the initialization script serializes quotes, backslashes, newlines,
U+2028, and U+2029 without producing executable JavaScript outside the assigned
string. Test that settings normalization keeps `overview`, canonicalizes provider
aliases, and clears invalid tabs.

- [ ] **Step 2: Verify the tests fail for the expected reasons**

```bash
cargo test --manifest-path src-tauri/Cargo.toml selected_tab
```

Expected: the script-serialization and normalization regressions fail.

- [ ] **Step 3: Implement the minimum fix**

Serialize the tab with `serde_json::to_string`, escape literal U+2028/U+2029 in
the serialized JSON, and embed that JSON value directly in the assignment.
Extend `MochiSettings::normalize_provider_ids` to normalize `selected_tab`, and
normalize settings before every persisted Rust update.

- [ ] **Step 4: Verify the focused tests pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml selected_tab
```

Expected: all selected-tab Rust tests pass.

### Task B: Persist only the selected tab

**Files:**

- Modify: `src-tauri/src/settings/commands.rs`
- Modify: `src-tauri/src/settings/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/tauri/commands/commands.ts`
- Modify: `src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.ts`
- Modify: `src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.test.ts`

- [ ] **Step 1: Add failing Rust and frontend regression tests**

The Rust test must prove a selected-tab update preserves unrelated settings that
changed after a window loaded its snapshot. The frontend test must prove tab
persistence sends only the new tab rather than a stale `MochiSettings` object.

- [ ] **Step 2: Verify the tests fail for the expected reasons**

```bash
cargo test --manifest-path src-tauri/Cargo.toml selected_tab
pnpm test -- src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.test.ts
```

Expected: the missing tab-only command causes both regressions to fail.

- [ ] **Step 3: Implement the minimum fix**

Add a `save_selected_tab` command that locks the current Rust settings, changes
only `selected_tab`, normalizes it, persists the result, and returns the current
settings. Register the command and call it from `persistTabChangeSettings`.

- [ ] **Step 4: Verify focused and full validation**

```bash
pnpm lint
pnpm format:check
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: all commands exit successfully.

---

### Task 1: Add `selected_tab` to `MochiSettings` (Rust)

**Files:**

- Modify: `src-tauri/src/settings/mod.rs`

- [ ] **Step 1: Add the field**

Add after `provider_configs` in the `MochiSettings` struct (around line 135):

```rust
    #[serde(default)]
    pub provider_configs: HashMap<String, ProviderConfig>,
    /// Tray panel / widget selected tab, persisted across windows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_tab: Option<String>,
```

- [ ] **Step 2: Update the Default impl**

Update the `Default` impl (around line 143) to include the new field:

```rust
impl Default for MochiSettings {
    fn default() -> Self {
        Self {
            update_channel: UpdateChannel::Stable,
            refresh_interval_seconds: 300,
            enabled_providers: Vec::new(),
            show_notifications: true,
            provider_configs: HashMap::new(),
            selected_tab: None,
        }
    }
}
```

- [ ] **Step 3: Run Rust compilation check**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/settings/mod.rs
git commit -m "feat(settings): add selected_tab field to MochiSettings"
```

---

### Task 2: Add initialization_script to widget creation (Rust)

**Files:**

- Modify: `src-tauri/src/widget/commands.rs`

- [ ] **Step 1: Add import for SettingsState**

Add after the existing `use crate::diagnostics::DiagnosticsState;` (around line 3):

```rust
use crate::settings::SettingsState;
```

- [ ] **Step 2: Add initialization_script to `build_widget_window`**

Replace `build_widget_window` (lines 74-88) with:

```rust
fn build_widget_window(app: &AppHandle) -> Result<WebviewWindow, tauri::Error> {
    let selected_tab = app
        .try_state::<SettingsState>()
        .and_then(|state| state.current().ok())
        .and_then(|s| s.selected_tab.clone())
        .unwrap_or_default();

    let init_script = if selected_tab.is_empty() {
        String::new()
    } else {
        format!(
            "window.__MOCHI_SELECTED_TAB__ = '{}';",
            selected_tab
        )
    };

    let mut builder = tauri::WebviewWindowBuilder::new(app, WIDGET_LABEL, app_shell_url())
        .title("Mochi Widget")
        .inner_size(WIDGET_WIDTH, 420.0)
        .min_inner_size(WIDGET_MIN_WIDTH, WIDGET_MIN_HEIGHT)
        .decorations(true)
        .resizable(true)
        .visible(matches!(
            crate::window_policy::decorated_window_initial_visibility(),
            crate::window_policy::DecoratedWindowInitialVisibility::Visible
        ));

    if !init_script.is_empty() {
        builder = builder.initialization_script(&init_script);
    }

    builder.build()
}
```

- [ ] **Step 3: Run Rust compilation check**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/widget/commands.rs
git commit -m "feat(widget): inject selected_tab via initialization_script on window creation"
```

---

### Task 3: Emit `"set-tab"` on widget re-show (Rust)

**Files:**

- Modify: `src-tauri/src/widget/commands.rs`

- [ ] **Step 1: Add "set-tab" emit at the start of `show_widget`**

Add after line 105 (`let window = ensure_widget_window(&app)?;`) and before `let policy = ...`:

```rust
    // Emit current selected tab before showing (only for reused windows)
    // First creation handles this via initialization_script.
    if let Some(state) = app.try_state::<SettingsState>() {
        if let Ok(settings) = state.current() {
            if let Some(tab) = &settings.selected_tab {
                let _ = app.emit_to(WIDGET_LABEL, "set-tab", tab);
            }
        }
    }
```

Note: `ensure_widget_window` may create the window on first call. For first creation, the `initialization_script` (from Task 2) handles the tab. The `emit_to` only matters for reused windows (re-show), where the frontend is already loaded and listening.

- [ ] **Step 2: Run Rust compilation check**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/widget/commands.rs
git commit -m "feat(widget): emit set-tab event before showing reused widget window"
```

---

### Task 4: Update tray-ui-store to read `window.__MOCHI_SELECTED_TAB__` (TypeScript)

**Files:**

- Modify: `src/features/tray/lib/stores/tray-ui-store/tray-ui-store.ts`

- [ ] **Step 1: Replace `readStoredTab` implementation**

Replace the entire `readStoredTab` function (lines 11-31) with:

```typescript
function readStoredTab(): TraySelectedTab {
  // Read from Rust-injected initialization script (synchronous, no IPC)
  const initialTab = (window as unknown as Record<string, unknown>).__MOCHI_SELECTED_TAB__;
  if (typeof initialTab === "string") {
    if (initialTab === "overview") return "overview";
    const parsed = ProviderIdSchema.safeParse(initialTab);
    if (parsed.success) return parsed.data;
  }

  return "overview";
}
```

- [ ] **Step 2: Remove localStorage functions**

Remove the following that are no longer needed:

- `STORAGE_KEY` constant (line 9)
- `persistTab` function (lines 33-43)
- `localStorage.getItem` / `setItem` usage

The `setSelectedTab` function in the store no longer needs to call `persistTab`:

```typescript
setSelectedTab: (tab) => {
  set({ selectedTab: tab });
},
```

- [ ] **Step 3: Run tests and lint**

```bash
pnpm lint
pnpm test -- src/features/tray/lib/stores/tray-ui-store/
```

Expected: tests pass (check if any tests rely on localStorage behavior).

- [ ] **Step 4: Commit**

```bash
git add src/features/tray/lib/stores/tray-ui-store/tray-ui-store.ts
git commit -m "feat(store): read selectedTab from window.__MOCHI_SELECTED_TAB__ instead of localStorage"
```

---

### Task 5: Save selected tab to settings on change (TypeScript)

**Files:**

- Modify: `src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.ts`

- [ ] **Step 1: Add the save call to `handleTabChange`**

Current `handleTabChange`:

```typescript
function handleTabChange(value: string) {
  const nextTab = parseTrayTabChange(value);
  setSelectedTab(nextTab);
  void syncTrayUsage(nextTab);
}
```

Replace with:

```typescript
function handleTabChange(value: string) {
  const nextTab = parseTrayTabChange(value);
  setSelectedTab(nextTab);
  void syncTrayUsage(nextTab);

  // Persist to shared settings (both windows read same settings.json)
  if (settings) {
    const updated = { ...settings, selected_tab: nextTab };
    void saveSettings(updated).then(() => {
      queryClient.setQueryData(queryKeys.settings, updated);
    });
  }
}
```

- [ ] **Step 2: Add the needed imports**

Add to the imports:

```typescript
import { useQueryClient } from "@tanstack/react-query";
import { saveSettings } from "@/lib/tauri/commands";
import { queryKeys } from "@/lib/query/keys";
```

Remove `syncTrayUsage` from `@/lib/tauri/commands` if it's no longer used elsewhere in this file (keep it if `handleRefreshProvider` also uses it — it does).

- [ ] **Step 3: Run lint and tests**

```bash
pnpm lint
pnpm test
```

- [ ] **Step 4: Commit**

```bash
git add src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.ts
git commit -m "feat(state): persist selectedTab to shared settings on tab change"
```

---

### Task 6: Add `"set-tab"` listener (TypeScript)

**Files:**

- Modify: `src/features/tray/hooks/use-tray-events/use-tray-events.ts`

- [ ] **Step 1: Add the listener**

Add after the `"tray-check-update"` listener (around line 90) and before `"app-navigate"`:

```typescript
      listen<string>("set-tab", (event) => {
        const tab = parseTrayTabChange(event.payload);
        useTrayUiStore.getState().setSelectedTab(tab);
      }),
```

- [ ] **Step 2: Add imports**

Add:

```typescript
import { parseTrayTabChange } from "@/lib/utils/tray-tab-selection";
import { useTrayUiStore } from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
```

- [ ] **Step 3: Run lint and tests**

```bash
pnpm lint
pnpm test
```

- [ ] **Step 4: Commit**

```bash
git add src/features/tray/hooks/use-tray-events/use-tray-events.ts
git commit -m "feat(events): add set-tab listener for widget re-show tab sync"
```

---

### Task 7: Make `"set-tab"` listener available in widget window

**Files:**

- Modify: `src/features/widget/components/widget-window/widget-window.tsx`
- Or check: `src/features/tray/components/tray-event-bridge/tray-event-bridge.tsx`

- [ ] **Step 1: Verify listener mounting**

The `useTrayEvents` hook is already called from `TrayEventBridge`, which is mounted by `RootComponent` in ALL windows (including the widget). So the `"set-tab"` listener added in Task 6 is already wired.

No code changes needed for this step — just verify.

- [ ] **Step 2: Run full validation**

```bash
pnpm build
pnpm lint
pnpm test
```

- [ ] **Step 3: Commit**

```bash
git commit -m "chore: verify set-tab listener wiring across all windows"
```
