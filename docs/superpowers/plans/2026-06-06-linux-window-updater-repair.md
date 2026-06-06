# Linux Window and Updater Repair Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix Linux tray/widget/settings parity, preserve selected-provider tray percentages, clean release-note rendering, and publish working signed updater feeds for installed Mochi builds.

**Architecture:** Keep Rust responsible for native tray/window/updater mechanics and React responsible for view state, scroll behavior, release-note rendering, and typed Tauri command calls. Add small pure helpers around tray selection, release-note sanitization, and updater-feed generation so the risky behavior is testable before it touches platform APIs.

**Tech Stack:** Tauri v2, Rust stable, React 19, TanStack Query 5, Zustand 5, Zod 4, Tailwind CSS 4, Vitest, Node scripts, GitHub Actions, oxlint/oxfmt.

---

## File Map

Modify Rust tray/window files:

- `src-tauri/src/tray/mod.rs`: native tray menu labels/order, checked channel menu items, checked-state sync command, command registration export.
- `src-tauri/src/linux_window_controls.rs`: structured diagnostics for Linux decoration/resizable setup.
- `src-tauri/src/widget/commands.rs`: widget window size/source diagnostics and Linux decoration reporting.
- `src-tauri/src/tray/panel.rs`: settings window Linux decoration reporting and optional app-window diagnostics.
- `src-tauri/src/updater/mod.rs`: exact updater endpoint URL tests for stable/unstable channels and unsupported-channel rejection.
- `src-tauri/src/lib.rs`: register any new Tauri command.
- `src-tauri/tauri.conf.json`: widget window defaults, updater artifact creation, updater pubkey placeholder handling.

Modify frontend tray/window files:

- `src/lib/tauri/commands.ts`: typed wrappers for new native tray channel command and current-selection sync helper if placed here.
- `src/lib/stores/tray-ui-store.ts`: expose pure selected-tab validity helper and current-tab sync helper.
- `src/hooks/use-cold-start-provider-refresh.ts`: sync native tray with current selected tab after boot refresh.
- `src/hooks/use-tray-events.ts`: sync native tray and channel check state after refresh/settings save.
- `src/hooks/use-tray-usage-sync.ts`: use the shared sync helper.
- `src/components/layout/root-component.tsx`: full-height shell class handling for Linux app/tray/widget windows.
- `src/components/tray/scroll-fade-region.tsx`: native vertical scroll mode with no overlay controls.
- `src/components/tray/tray-panel-shell.tsx`: use native vertical scroll mode where needed.
- `src/components/settings/settings-form.tsx`: remove `LinuxTrayHint`; use safer scroll mode.
- `src/components/updates/update-page-content.tsx`: scroll notes by overflow, not section count.
- `src/components/updates/release-notes-dialog.tsx`: always cap and scroll overflowing notes.
- `src/components/widget/widget-window.tsx`: remove redundant outer background/padding.
- `src/lib/utils/tray-panel-layout.ts`: widget/panel shell class helpers if needed.

Modify updater/release-note files:

- `src/lib/updates/sanitize-release-notes.ts`: new sanitizer helper.
- `src/lib/updates/sanitize-release-notes.test.ts`: sanitizer test coverage.
- `src/lib/query/update-check.ts`: sanitize updater manifest notes before caching and returning.
- `src/lib/query/update-check.test.ts`: assert sanitized manifest notes.
- `src/lib/updates/current-release-notes.ts`: sanitize GitHub fallback notes before caching.
- `src/lib/updates/current-release-notes.test.ts`: assert fallback sanitization and installed-version labeling behavior.
- `src/lib/updates/release-notes-cache.ts`: sanitize legacy cached reads and store note source metadata.
- `src/components/settings/settings-update-section.tsx`, `src/components/updates/update-page.tsx`: consume sanitized cached notes and label installed-version fallback notes honestly.

Create release feed files:

- `scripts/release/build-updater-feed.mjs`: build versioned stable/unstable updater JSON feed tree from signed artifacts.
- `scripts/release/build-updater-feed.test.mjs`: Node test for endpoint/platform mapping, backfill versions, and required fields.
- `scripts/release/validate-updater-feed.mjs`: curl/read validation helper for CI or local artifact directories.

Modify workflows:

- `.github/workflows/release-stable.yml`: create updater artifacts, inject pubkey, build/publish stable feed, validate endpoints.
- `.github/workflows/release-unstable.yml`: create updater artifacts, inject pubkey, build/publish unstable feed, validate endpoints.

---

### Task 1: Native Tray Menu and Update Channel State

**Files:**

- Modify: `src-tauri/src/tray/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/tauri/commands.ts`
- Modify: `src/hooks/use-tray-events.ts`
- Test: Rust tests in `src-tauri/src/tray/mod.rs`
- Test: `src/hooks/use-tray-events.test.ts`

- [ ] **Step 1: Add failing Rust tests for tray menu model**

In `src-tauri/src/tray/mod.rs`, add tests for a not-yet-existing `build_tray_menu_model` helper. Do not add the helper in this step; the tests must fail because production menu construction has not been extracted yet.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
enum TrayMenuEntry {
    Item { id: &'static str, label: &'static str },
    Channel { id: &'static str, label: &'static str, checked: bool },
    Submenu { label: &'static str, children: Vec<TrayMenuEntry> },
    Separator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrayMenuModel {
    entries: Vec<TrayMenuEntry>,
}
```

Add tests:

```rust
#[test]
fn tray_menu_model_removes_show_usage_and_prioritizes_widget() {
    let model = build_tray_menu_model("stable");
    let labels = tray_menu_labels(&model);
    assert_eq!(labels.first(), Some(&"Open widget"));
    assert!(!labels.contains(&"Show usage"));
    assert!(!labels.contains(&"Show widget"));
    assert!(labels.contains(&"Refresh usage"));
    assert!(labels.contains(&"Settings"));
    assert!(labels.contains(&"Update channel"));
}

#[test]
fn tray_menu_model_marks_current_channel() {
    assert_eq!(checked_channel_id(&build_tray_menu_model("stable")), Some("channel-stable"));
    assert_eq!(
        checked_channel_id(&build_tray_menu_model("unstable")),
        Some("channel-unstable")
    );
    assert_eq!(checked_channel_id(&build_tray_menu_model("unexpected")), Some("channel-stable"));
}
```

Add these test-only helpers in the Rust test module:

```rust
fn tray_menu_labels(model: &TrayMenuModel) -> Vec<&'static str> {
    fn collect(entry: &TrayMenuEntry, labels: &mut Vec<&'static str>) {
        match entry {
            TrayMenuEntry::Item { label, .. }
            | TrayMenuEntry::Channel { label, .. }
            | TrayMenuEntry::Submenu { label, .. } => labels.push(label),
            TrayMenuEntry::Separator => {}
        }

        if let TrayMenuEntry::Submenu { children, .. } = entry {
            for child in children {
                collect(child, labels);
            }
        }
    }

    let mut labels = Vec::new();
    for entry in &model.entries {
        collect(entry, &mut labels);
    }
    labels
}

fn checked_channel_id(model: &TrayMenuModel) -> Option<&'static str> {
    model.entries.iter().find_map(|entry| match entry {
        TrayMenuEntry::Submenu { children, .. } => children.iter().find_map(|child| match child {
            TrayMenuEntry::Channel { id, checked: true, .. } => Some(*id),
            _ => None,
        }),
        _ => None,
    })
}
```

- [ ] **Step 2: Run Rust tray tests and verify failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml tray_menu_model
```

Expected before implementation: tests fail because `build_tray_menu_model` is missing.

- [ ] **Step 3: Implement checked native channel menu**

In `src-tauri/src/tray/mod.rs`:

- Import checked menu types:

```rust
use tauri::menu::{CheckMenuItem, CheckMenuItemBuilder, Menu, MenuItem, PredefinedMenuItem, Submenu};
```

- Add the production menu model that `setup_tray` will consume:

```rust
fn build_tray_menu_model(channel: &str) -> TrayMenuModel {
    let unstable = channel == "unstable";
    TrayMenuModel {
        entries: vec![
            TrayMenuEntry::Item { id: "widget", label: "Open widget" },
            TrayMenuEntry::Item { id: "refresh", label: "Refresh usage" },
            TrayMenuEntry::Item { id: "settings", label: "Settings" },
            TrayMenuEntry::Submenu {
                label: "Update channel",
                children: vec![
                    TrayMenuEntry::Channel {
                        id: "channel-stable",
                        label: "Stable",
                        checked: !unstable,
                    },
                    TrayMenuEntry::Channel {
                        id: "channel-unstable",
                        label: "Unstable",
                        checked: unstable,
                    },
                ],
            },
            TrayMenuEntry::Item { id: "update", label: "Check for updates" },
            TrayMenuEntry::Separator,
            TrayMenuEntry::Item { id: "quit", label: "Quit Mochi" },
        ],
    }
}
```

- Add state to retain handles:

```rust
#[derive(Clone)]
pub struct TrayChannelMenuState {
    stable: CheckMenuItem,
    unstable: CheckMenuItem,
}

impl TrayChannelMenuState {
    fn set_channel(&self, channel: &str) -> Result<(), String> {
        let unstable = channel == "unstable";
        self.stable.set_checked(!unstable).map_err(|error| error.to_string())?;
        self.unstable
            .set_checked(unstable)
            .map_err(|error| error.to_string())
    }
}
```

- Change `setup_tray` to read settings and use checked items:

```rust
let current_channel = app
    .try_state::<SettingsState>()
    .and_then(|state| state.current().ok())
    .map(|settings| settings.update_channel.to_string())
    .unwrap_or_else(|| "stable".to_string());
let model = build_tray_menu_model(&current_channel);
let checked_channel = if current_channel == "unstable" {
    "unstable"
} else {
    "stable"
};

let widget_item = MenuItem::with_id(app, "widget", "Open widget", true, None::<&str>)?;
let refresh_item = MenuItem::with_id(app, "refresh", "Refresh usage", true, None::<&str>)?;
let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
let stable_channel_item = CheckMenuItemBuilder::with_id("channel-stable", "Stable")
    .checked(checked_channel == "stable")
    .build(app)?;
let unstable_channel_item = CheckMenuItemBuilder::with_id("channel-unstable", "Unstable")
    .checked(checked_channel == "unstable")
    .build(app)?;
app.manage(TrayChannelMenuState {
    stable: stable_channel_item.clone(),
    unstable: unstable_channel_item.clone(),
});
```

- Build the native `Menu` from `model.entries`, so the tested model is the source of truth for setup. Remove `usage_item` from the menu and remove the `"usage"` event branch.
- Add command:

```rust
#[tauri::command]
pub fn sync_tray_update_channel(
    channel: String,
    state: State<'_, TrayChannelMenuState>,
) -> Result<(), String> {
    state.set_channel(channel.as_str())
}
```

- Export/register `sync_tray_update_channel` in `src-tauri/src/lib.rs`.

- [ ] **Step 4: Add frontend wrapper and settings-save sync**

In `src/lib/tauri/commands.ts`:

```ts
export function syncTrayUpdateChannel(channel: UpdateChannel): Promise<void> {
  return invoke<void>("sync_tray_update_channel", { channel });
}
```

In `src/hooks/use-tray-events.ts`, after settings save success in the `tray-set-channel` listener and in `reconcileSettingsSaveSuccess`, call `syncTrayUpdateChannel(settings.update_channel)`.

Use dependency injection for the existing test:

```ts
export async function reconcileSettingsSaveSuccess(
  queryClient: SettingsSaveSuccessQueryClient,
  settings: MochiSettings,
  syncUsage: () => Promise<void> = syncTrayUsage,
  syncChannel: (channel: UpdateChannel) => Promise<void> = syncTrayUpdateChannel,
): Promise<void> {
  queryClient.setQueryData(queryKeys.settings, settings);
  await syncChannel(settings.update_channel);
  await queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots });
  await syncUsage();
}
```

- [ ] **Step 5: Update tests and verify pass**

In `src/hooks/use-tray-events.test.ts`, update the existing call-order test to expect channel sync before usage sync:

```ts
await reconcileSettingsSaveSuccess(
  queryClient,
  DEFAULT_MOCHI_SETTINGS,
  () => {
    calls.push("sync-usage");
    return Promise.resolve();
  },
  (channel) => {
    calls.push(`sync-channel:${channel}`);
    return Promise.resolve();
  },
);

expect(calls).toEqual([
  `set:${queryKeys.settings.join("/")}`,
  "sync-channel:stable",
  `invalidate:${queryKeys.usageSnapshots.join("/")}`,
  "sync-usage",
]);
```

Run:

```bash
pnpm test src/hooks/use-tray-events.test.ts
cargo test --manifest-path src-tauri/Cargo.toml tray_menu_model
```

Expected: both commands pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/tray/mod.rs src-tauri/src/lib.rs src/lib/tauri/commands.ts src/hooks/use-tray-events.ts src/hooks/use-tray-events.test.ts
git commit -m "fix(tray): sync native menu state"
```

---

### Task 2: Preserve Selected Provider Tray Percentage

**Files:**

- Modify: `src/lib/stores/tray-ui-store.ts`
- Modify: `src/hooks/use-cold-start-provider-refresh.ts`
- Modify: `src/hooks/use-tray-events.ts`
- Modify: `src/hooks/use-tray-usage-sync.ts`
- Test: `src/lib/stores/tray-ui-store.test.ts` or new `src/lib/stores/tray-ui-store.test.ts`
- Test: `src/hooks/use-tray-events.test.ts`
- Test: `src/hooks/use-cold-start-provider-refresh.test.ts`

- [ ] **Step 1: Write failing selected-tab validity tests**

Create `src/lib/stores/tray-ui-store.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import { resolveValidTraySelection } from "./tray-ui-store";

describe("resolveValidTraySelection", () => {
  it("keeps an enabled provider selected even when snapshots are missing", () => {
    expect(resolveValidTraySelection("codex", ["codex"])).toBe("codex");
  });

  it("falls back to overview when selected provider is disabled", () => {
    expect(resolveValidTraySelection("codex", ["cursor"])).toBe("overview");
  });

  it("keeps overview selected", () => {
    expect(resolveValidTraySelection("overview", ["codex"])).toBe("overview");
  });
});
```

- [ ] **Step 2: Run selected-tab tests and verify failure**

Run:

```bash
pnpm test src/lib/stores/tray-ui-store.test.ts
```

Expected: fails because `resolveValidTraySelection` does not exist.

- [ ] **Step 3: Add selected-tab sync helper**

In `src/lib/stores/tray-ui-store.ts`:

```ts
import { syncTrayUsage } from "@/lib/tauri/commands";
```

Add exports:

```ts
export function resolveValidTraySelection(
  selected: TraySelectedTab,
  enabledProviders: ProviderId[],
): TraySelectedTab {
  if (selected === "overview") {
    return "overview";
  }

  return enabledProviders.includes(selected) ? selected : "overview";
}

export function currentTraySelection(): TraySelectedTab {
  return useTrayUiStore.getState().selectedTab;
}

export function syncCurrentTrayUsage(
  settings: Pick<MochiSettings, "enabled_providers">,
): Promise<void> {
  const selected = currentTraySelection();
  const validSelection = resolveValidTraySelection(selected, settings.enabled_providers);
  if (validSelection !== selected) {
    useTrayUiStore.getState().setSelectedTab(validSelection);
  }
  return syncTrayUsage(validSelection);
}
```

- [ ] **Step 4: Replace selection-less sync calls**

In `src/hooks/use-cold-start-provider-refresh.ts`, replace `syncTrayUsage()` with `syncCurrentTrayUsage(settings)` and extract the refresh sequence so it can be tested without rendering React:

```ts
export async function runColdStartProviderRefreshSequence(
  settings: MochiSettings,
  refresh: () => Promise<unknown>,
  invalidate: () => Promise<unknown>,
  syncUsage: (settings: Pick<MochiSettings, "enabled_providers">) => Promise<unknown>,
) {
  await refresh();
  await invalidate();
  await syncUsage(settings);
}
```

In `src/hooks/use-tray-events.ts`, replace tray-refresh and settings-save default usage sync with `syncCurrentTrayUsage(settings)`. For the native `tray-refresh` listener, read current settings from `queryClient.getQueryData<MochiSettings>(queryKeys.settings)` and fall back to `DEFAULT_MOCHI_SETTINGS` only if the query cache is still empty. Never call `syncTrayUsage()` with `undefined` selection from refresh paths.

Update `reconcileSettingsSaveSuccess` so the usage-sync dependency receives the saved settings:

```ts
export async function reconcileSettingsSaveSuccess(
  queryClient: SettingsSaveSuccessQueryClient,
  settings: MochiSettings,
  syncUsage: (
    settings: Pick<MochiSettings, "enabled_providers">,
  ) => Promise<void> = syncCurrentTrayUsage,
  syncChannel: (channel: UpdateChannel) => Promise<void> = syncTrayUpdateChannel,
): Promise<void> {
  queryClient.setQueryData(queryKeys.settings, settings);
  await syncChannel(settings.update_channel);
  await queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots });
  await syncUsage(settings);
}
```

Extract a testable tray refresh sequence:

```ts
export async function runTrayRefreshEventSequence(
  queryClient: SettingsSaveSuccessQueryClient,
  settings: MochiSettings,
  refresh: () => Promise<unknown>,
  syncUsage: (settings: Pick<MochiSettings, "enabled_providers">) => Promise<unknown>,
) {
  await refresh().catch(() => undefined);
  await queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots });
  await syncUsage(settings);
}
```

In `src/hooks/use-tray-usage-sync.ts`, read settings with `useSettings()` and call `syncCurrentTrayUsage(settings)` in the data-success effect. Keep the selected-tab dependency so React still resyncs when tab changes.

- [ ] **Step 5: Add tests for selected provider argument, not just wrapper call**

In `src/lib/stores/tray-ui-store.test.ts`, mock the native command and assert the selected provider is the command argument:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";

import { syncTrayUsage } from "@/lib/tauri/commands";

import { resolveValidTraySelection, syncCurrentTrayUsage, useTrayUiStore } from "./tray-ui-store";

vi.mock("@/lib/tauri/commands", () => ({
  syncTrayUsage: vi.fn(() => Promise.resolve()),
}));

beforeEach(() => {
  vi.mocked(syncTrayUsage).mockClear();
  useTrayUiStore.getState().setSelectedTab("overview");
});

it("passes the current selected provider to the native tray sync", async () => {
  useTrayUiStore.getState().setSelectedTab("codex");

  await syncCurrentTrayUsage({ enabled_providers: ["codex"] });

  expect(syncTrayUsage).toHaveBeenCalledWith("codex");
});

it("passes overview only when the selected provider is disabled", async () => {
  useTrayUiStore.getState().setSelectedTab("codex");

  await syncCurrentTrayUsage({ enabled_providers: ["cursor"] });

  expect(syncTrayUsage).toHaveBeenCalledWith("overview");
  expect(useTrayUiStore.getState().selectedTab).toBe("overview");
});
```

In `src/hooks/use-tray-events.test.ts`, add a direct assertion for the real tray refresh sequence. This must not inject a callback that hard-codes `"codex"`; it must use `syncCurrentTrayUsage` so the stored selected tab is read:

```ts
import { syncTrayUsage } from "@/lib/tauri/commands";
import { syncCurrentTrayUsage, useTrayUiStore } from "@/lib/stores/tray-ui-store";

import { runTrayRefreshEventSequence } from "./use-tray-events";

vi.mock("@/lib/tauri/commands", () => ({
  syncTrayUsage: vi.fn(() => Promise.resolve()),
}));

it("native tray refresh syncs the selected provider from the store", async () => {
  useTrayUiStore.getState().setSelectedTab("codex");
  const queryClient = {
    invalidateQueries: () => Promise.resolve(),
  };

  await runTrayRefreshEventSequence(
    queryClient,
    { ...DEFAULT_MOCHI_SETTINGS, enabled_providers: ["codex"] },
    () => Promise.resolve(),
    syncCurrentTrayUsage,
  );

  expect(syncTrayUsage).toHaveBeenCalledWith("codex");
});

it("settings save syncs the selected provider from the store", async () => {
  useTrayUiStore.getState().setSelectedTab("codex");
  const queryClient = {
    setQueryData: () => undefined,
    invalidateQueries: () => Promise.resolve(),
  };

  await reconcileSettingsSaveSuccess(
    queryClient,
    { ...DEFAULT_MOCHI_SETTINGS, enabled_providers: ["codex"] },
    () => syncCurrentTrayUsage({ enabled_providers: ["codex"] }),
    () => Promise.resolve(),
  );

  expect(syncTrayUsage).toHaveBeenCalledWith("codex");
});
```

Add a disabled-provider fallback test in `src/lib/stores/tray-ui-store.test.ts`:

```ts
it("persists overview only when the stored provider is no longer enabled", async () => {
  expect(resolveValidTraySelection("codex", [])).toBe("overview");
  expect(resolveValidTraySelection("codex", ["codex"])).toBe("codex");
});
```

In `src/hooks/use-cold-start-provider-refresh.test.ts`, add:

```ts
import { syncTrayUsage } from "@/lib/tauri/commands";
import { syncCurrentTrayUsage, useTrayUiStore } from "@/lib/stores/tray-ui-store";

import { runColdStartProviderRefreshSequence } from "./use-cold-start-provider-refresh";

vi.mock("@/lib/tauri/commands", () => ({
  syncTrayUsage: vi.fn(() => Promise.resolve()),
}));

it("cold start refresh syncs the selected provider from the store after invalidating cache", async () => {
  useTrayUiStore.getState().setSelectedTab("codex");

  await runColdStartProviderRefreshSequence(
    { ...settings(["codex"]), enabled_providers: ["codex"] },
    () => Promise.resolve(),
    () => Promise.resolve(),
    syncCurrentTrayUsage,
  );

  expect(syncTrayUsage).toHaveBeenCalledWith("codex");
});
```

- [ ] **Step 6: Run tests**

```bash
pnpm test src/lib/stores/tray-ui-store.test.ts src/hooks/use-tray-events.test.ts src/hooks/use-cold-start-provider-refresh.test.ts
```

Expected: pass.

- [ ] **Step 7: Commit**

```bash
git add src/lib/stores/tray-ui-store.ts src/lib/stores/tray-ui-store.test.ts src/hooks/use-cold-start-provider-refresh.ts src/hooks/use-cold-start-provider-refresh.test.ts src/hooks/use-tray-events.ts src/hooks/use-tray-events.test.ts src/hooks/use-tray-usage-sync.ts
git commit -m "fix(tray): preserve selected provider"
```

---

### Task 3: Linux Root Layout and Native Vertical Scrolling

**Files:**

- Modify: `src/components/layout/root-component.tsx`
- Modify: `src/components/layout/root-component-state.ts`
- Create: `src/lib/tauri/widget-window.ts`
- Modify: `src/components/tray/scroll-fade-region.tsx`
- Modify: `src/components/tray/tray-panel-shell.tsx`
- Modify: `src/components/settings/settings-form.tsx`
- Modify: `src/components/updates/update-page-content.tsx`
- Modify: `src/components/updates/release-notes-dialog.tsx`
- Test: `src/components/layout/root-component-state.test.ts`
- Test: `src/components/tray/scroll-fade-region.test.tsx` or source-level test if the repo does not render components
- Test: `src/components/settings/settings-form.test.ts`

- [ ] **Step 1: Add failing root shell classification test**

In `src/components/layout/root-component-state.ts`, add pure helper signature:

```ts
export function shouldUseFullHeightWindowShell(input: {
  isTrayPanelWindow: boolean;
  isAppWindow: boolean;
  isWidgetWindow?: boolean;
  platform: PlatformId;
}): boolean {
  return false;
}
```

In `src/components/layout/root-component-state.test.ts`, add:

```ts
import { shouldUseFullHeightWindowShell } from "./root-component-state";

it("uses full-height shell classes for linux app and tray windows", () => {
  expect(
    shouldUseFullHeightWindowShell({
      platform: "linux",
      isTrayPanelWindow: true,
      isAppWindow: false,
    }),
  ).toBe(true);
  expect(
    shouldUseFullHeightWindowShell({
      platform: "linux",
      isTrayPanelWindow: false,
      isAppWindow: true,
    }),
  ).toBe(true);
});

it("uses full-height shell classes for linux widget windows", () => {
  expect(
    shouldUseFullHeightWindowShell({
      platform: "linux",
      isTrayPanelWindow: false,
      isAppWindow: false,
      isWidgetWindow: true,
    }),
  ).toBe(true);
});
```

- [ ] **Step 2: Run root shell test and verify failure**

```bash
pnpm test src/components/layout/root-component-state.test.ts
```

Expected: fails until helper returns true for Linux window shells.

- [ ] **Step 3: Implement full-height shell helper**

In `src/components/layout/root-component-state.ts`:

```ts
export function shouldUseFullHeightWindowShell({
  isTrayPanelWindow,
  isAppWindow,
  isWidgetWindow = false,
}: {
  isTrayPanelWindow: boolean;
  isAppWindow: boolean;
  isWidgetWindow?: boolean;
  platform: PlatformId;
}): boolean {
  return isTrayPanelWindow || isAppWindow || isWidgetWindow;
}
```

In `src/components/layout/root-component.tsx`, compute:

```ts
const isWidgetWindow = rootState.isWidgetWindow;
const isFullHeightWindowShell = shouldUseFullHeightWindowShell({
  isTrayPanelWindow,
  isAppWindow,
  isWidgetWindow,
  platform,
});
const supportsNativeWindowGlass = platform === "macos" || platform === "windows";
const isNativeGlassShell = supportsNativeWindowGlass && (isTrayPanelWindow || isAppWindow);
```

Create `src/lib/tauri/widget-window.ts`:

```ts
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

export const WIDGET_WINDOW_LABEL = "widget";

export function readIsWidgetWindow(): boolean {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return false;
  }

  try {
    return getCurrentWebviewWindow().label === WIDGET_WINDOW_LABEL;
  } catch {
    return false;
  }
}
```

Add `isWidgetWindow` to `HydrationSafeRootState`, initialize it to `false`, read it in `RootComponent` with `readIsWidgetWindow()`, and include it in the `setRootState` payload. Use `isFullHeightWindowShell` for `h-full`/flex wrapper classes and body overflow, but keep `bg-transparent` only for `isNativeGlassShell`.

- [ ] **Step 4: Add native vertical scroll mode**

In `src/components/tray/scroll-fade-region.tsx`, extend props:

```ts
interface ScrollFadeRegionProps {
  orientation: ScrollFadeOrientation;
  children: ReactNode;
  className?: string;
  scrollClassName?: string;
  rowHeightClassName?: string;
  fadeInset?: number;
  controls?: "fade" | "none";
  onCycleForward?: (scrollEl: HTMLDivElement) => void;
  onCycleBackward?: (scrollEl: HTMLDivElement) => void;
}
```

Only render `ScrollFadeEdgeOverlays` when `controls !== "none"`.

When `orientation === "vertical" && controls === "none"`, omit `scrollbar-none` and mask classes:

```ts
const shouldUseMask = controls !== "none";
const maskClass = shouldUseMask
  ? scrollFadeMaskClass(orientation, canScrollStart, canScrollEnd)
  : undefined;
```

Set default `controls = "fade"` to preserve current horizontal tabs.

- [ ] **Step 5: Use native vertical mode in affected windows**

Set `controls="none"` on vertical `ScrollFadeRegion` usages in:

- `src/components/tray/tray-panel-shell.tsx`
- `src/components/settings/settings-form.tsx`
- `src/components/updates/update-page-content.tsx`
- `src/components/updates/release-notes-dialog.tsx`

In `UpdatePageContent`, remove section-count gating:

```tsx
<ScrollFadeRegion orientation="vertical" controls="none" className="min-h-0 flex-1">
  <div className={`${trayPanelSpacing.contentX} py-2`}>{content}</div>
</ScrollFadeRegion>
```

In `ReleaseNotesDialog`, always use a scroll region for the body inside the capped dialog content.

- [ ] **Step 6: Remove settings Linux hint**

In `src/components/settings/settings-form.tsx`:

- Remove `import { LinuxTrayHint } from "./linux-tray-hint";`
- Remove `<LinuxTrayHint />`

Add source-level test in `src/components/settings/settings-form.test.ts`:

```ts
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

it("does not render the linux tray hint in the normal settings form", () => {
  const source = readFileSync(resolve("src/components/settings/settings-form.tsx"), "utf8");
  expect(source).not.toContain("LinuxTrayHint");
  expect(source).not.toContain("data-linux-tray-hint");
});
```

- [ ] **Step 7: Run focused frontend tests**

```bash
pnpm test src/components/layout/root-component-state.test.ts src/components/settings/settings-form.test.ts src/lib/utils/tray-panel-layout.test.ts
```

Expected: pass.

- [ ] **Step 8: Commit**

```bash
git add src/components/layout/root-component.tsx src/components/layout/root-component-state.ts src/components/layout/root-component-state.test.ts src/components/tray/scroll-fade-region.tsx src/components/tray/tray-panel-shell.tsx src/components/settings/settings-form.tsx src/components/settings/settings-form.test.ts src/components/updates/update-page-content.tsx src/components/updates/release-notes-dialog.tsx
git commit -m "fix(ui): restore linux scroll shells"
```

---

### Task 4: Widget Parity and Linux Window Diagnostics

**Files:**

- Modify: `src/components/widget/widget-window.tsx`
- Modify: `src/components/widget/widget-window.test.ts`
- Modify: `src/lib/utils/tray-panel-layout.ts`
- Modify: `src/lib/utils/tray-panel-layout.test.ts`
- Modify: `src-tauri/src/widget/commands.rs`
- Modify: `src-tauri/src/tray/panel.rs`
- Modify: `src-tauri/src/linux_window_controls.rs`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Add failing widget config/source tests**

In `src/components/widget/widget-window.test.ts`, add:

```ts
it("does not add an extra opaque outer widget surface", () => {
  const source = readFileSync(resolve("src/components/widget/widget-window.tsx"), "utf8");
  expect(source).not.toContain("bg-background flex h-full");
  expect(source).toContain("TrayPanelShell");
});
```

Add `scripts` or source-level config assertion in `src/components/widget/widget-window.test.ts`:

```ts
it("keeps tauri widget defaults aligned with rust widget builder", () => {
  const config = JSON.parse(readFileSync(resolve("src-tauri/tauri.conf.json"), "utf8"));
  const widget = config.app.windows.find((window: { label: string }) => window.label === "widget");
  expect(widget.width).toBe(360);
  expect(widget.height).toBe(420);
  expect(widget.minWidth).toBe(320);
  expect(widget.maxWidth).toBe(480);
});
```

- [ ] **Step 2: Run widget tests and verify failure**

```bash
pnpm test src/components/widget/widget-window.test.ts
```

Expected: fails until widget wrapper/config are updated.

- [ ] **Step 3: Update widget frontend shell**

In `src/components/widget/widget-window.tsx`, change the outer wrapper to avoid a second opaque surface:

```tsx
return (
  <div className="flex h-full min-h-0 flex-col overflow-hidden">
    <TrayPanelShell layoutRef={layoutRef}>
      <section
        data-tray-panel-content
        className="mx-auto flex w-full max-w-[360px] min-w-0 flex-col"
      >
        <UsageSnapshotsPanel
          error={error}
          isError={isError}
          isPending={isPending}
          isSuccess={isSuccess}
          enabledProviderCount={settings?.enabled_providers.length ?? 0}
          activeTab={selectedTab}
          onTabChange={handleTabChange}
          tabs={tabs}
          states={states}
          onRefreshProvider={handleRefreshProvider}
          refreshingProvider={refreshingProvider}
        />
        <TrayPanelDivider inset data-tray-panel-separator />
        <TrayPanelFooter
          isRefreshing={isFetching || refreshProviderMutation.isPending || isRefreshingAll}
          onRefresh={() => {
            void refreshAll();
          }}
          onQuit={() => {
            void quitApp();
          }}
        />
      </section>
    </TrayPanelShell>
  </div>
);
```

Keep content width capped at `max-w-[360px]`.

- [ ] **Step 4: Align widget native defaults**

In `src-tauri/tauri.conf.json`, change widget window:

```json
{
  "label": "widget",
  "title": "Mochi Widget",
  "url": "index.html",
  "width": 360,
  "height": 420,
  "minWidth": 320,
  "maxWidth": 480,
  "minHeight": 200,
  "visible": false,
  "alwaysOnTop": true,
  "resizable": true
}
```

In `src-tauri/src/widget/mod.rs`, set `WIDGET_MIN_WIDTH` to `320.0` if it is still `280.0`.

In `src-tauri/src/widget/commands.rs`, align builder `.inner_size(360.0, 420.0)` and fallback width `unwrap_or(360.0)`.

- [ ] **Step 5: Add Linux decoration diagnostics**

In `src-tauri/src/linux_window_controls.rs`, replace `prepare_decorated_window` body with a result-returning helper:

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct LinuxWindowControlDiagnostics {
    pub label: String,
    pub platform: String,
    pub creation_source: String,
    pub decorations_action: String,
    pub decorations_error: Option<String>,
    pub decorations_ok: bool,
    pub resizable_action: String,
    pub resizable_error: Option<String>,
    pub resizable_ok: bool,
}

pub fn prepare_decorated_window(
    window: &WebviewWindow,
    label: &str,
    creation_source: &str,
) -> LinuxWindowControlDiagnostics {
    #[cfg(target_os = "linux")]
    {
        let decorations_result = window.set_decorations(true);
        let resizable_result = window.set_resizable(true);
        crate::diagnostics::log_window_action_result(label, "linux_set_decorations", decorations_result.as_ref().map(|_| ()));
        crate::diagnostics::log_window_action_result(label, "linux_set_resizable", resizable_result.as_ref().map(|_| ()));
        return LinuxWindowControlDiagnostics {
            label: label.to_string(),
            platform: "linux".to_string(),
            creation_source: creation_source.to_string(),
            decorations_action: "linux_set_decorations".to_string(),
            decorations_error: decorations_result.as_ref().err().map(ToString::to_string),
            decorations_ok: decorations_result.is_ok(),
            resizable_action: "linux_set_resizable".to_string(),
            resizable_error: resizable_result.as_ref().err().map(ToString::to_string),
            resizable_ok: resizable_result.is_ok(),
        };
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = window;
        LinuxWindowControlDiagnostics {
            label: label.to_string(),
            platform: std::env::consts::OS.to_string(),
            creation_source: creation_source.to_string(),
            decorations_action: "native_decorations_default".to_string(),
            decorations_error: None,
            decorations_ok: true,
            resizable_action: "native_resizable_default".to_string(),
            resizable_error: None,
            resizable_ok: true,
        }
    }
}
```

Callers must pass `creation_source` as `"tauri-config"` when reusing the configured widget window and `"rust-builder"` when creating from `WebviewWindowBuilder`. Record the returned struct in `DiagnosticsState` using a new method such as `record_linux_window_controls(diagnostics)`, and add a unit test that serializes `LinuxWindowControlDiagnostics` with `creation_source`, action names, and raw error fields.

- [ ] **Step 6: Run focused tests**

```bash
pnpm test src/components/widget/widget-window.test.ts src/lib/utils/tray-panel-layout.test.ts
cargo test --manifest-path src-tauri/Cargo.toml widget
```

Expected: pass.

- [ ] **Step 7: Commit**

```bash
git add src/components/widget/widget-window.tsx src/components/widget/widget-window.test.ts src/lib/utils/tray-panel-layout.ts src/lib/utils/tray-panel-layout.test.ts src-tauri/src/widget/commands.rs src-tauri/src/widget/mod.rs src-tauri/src/tray/panel.rs src-tauri/src/linux_window_controls.rs src-tauri/tauri.conf.json
git commit -m "fix(widget): align linux window shell"
```

---

### Task 5: Release Notes Sanitization and Scrollable Dialog

**Files:**

- Create: `src/lib/updates/sanitize-release-notes.ts`
- Create: `src/lib/updates/sanitize-release-notes.test.ts`
- Modify: `src/lib/query/update-check.ts`
- Modify: `src/lib/query/update-check.test.ts`
- Modify: `src/lib/updates/current-release-notes.ts`
- Modify: `src/lib/updates/current-release-notes.test.ts`
- Modify: `src/lib/updates/release-notes-cache.ts`
- Modify: `src/lib/updates/release-notes-cache.test.ts`
- Modify: `src/components/settings/settings-update-section.tsx`
- Modify: `src/components/updates/update-page.tsx`
- Modify: `src/components/updates/release-notes-dialog.tsx`
- Modify: `src/components/updates/update-page-content.tsx`

- [ ] **Step 1: Write failing sanitizer tests**

Create `src/lib/updates/sanitize-release-notes.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import { sanitizeReleaseNotesForApp } from "./sanitize-release-notes";

describe("sanitizeReleaseNotesForApp", () => {
  it("keeps change sections and removes install commands", () => {
    const notes = [
      "## Mochi v0.2.0",
      "",
      "### What's changed",
      "- SQLite usage persistence",
      "- Cached-usage CLI",
      "",
      "### Install stable",
      "- macOS: `curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-macos.sh | bash`",
      "- Linux: `curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-linux.sh | bash`",
    ].join("\n");

    expect(sanitizeReleaseNotesForApp(notes)).toBe(
      "### What's changed\n- SQLite usage persistence\n- Cached-usage CLI",
    );
  });

  it("drops artifact lists and binary file names", () => {
    const notes = [
      "### Fixes",
      "- Fix tray scrolling",
      "",
      "### Assets",
      "- Mochi_0.2.0_amd64.AppImage",
      "- Mochi_0.2.0_x64.dmg",
    ].join("\n");

    expect(sanitizeReleaseNotesForApp(notes)).toBe("### Fixes\n- Fix tray scrolling");
  });
});
```

- [ ] **Step 2: Run sanitizer tests and verify failure**

```bash
pnpm test src/lib/updates/sanitize-release-notes.test.ts
```

Expected: fails because the sanitizer file does not exist.

- [ ] **Step 3: Implement sanitizer helper**

Create `src/lib/updates/sanitize-release-notes.ts`:

```ts
const KEEP_SECTION = /^(what'?s changed|changes|fixes|features|improvements|bug fixes)$/i;
const DROP_SECTION =
  /^(install|install stable|install unstable|downloads?|assets?|binaries|packages?)$/i;
const ARTIFACT_LINE = /\.(appimage|deb|rpm|dmg|msi|exe|app\.tar\.gz)\b/i;
const COMMAND_LINE = /\b(curl|irm|bash|powershell|brew install|sudo)\b/i;

function headingTitle(line: string): string | null {
  return /^#{1,6}\s+(.+)$/.exec(line.trim())?.[1]?.trim() ?? null;
}

export function sanitizeReleaseNotesForApp(notes: string | null | undefined): string {
  if (!notes?.trim()) {
    return "";
  }

  const output: string[] = [];
  let keeping = false;
  let sawKeptHeading = false;

  for (const rawLine of notes.split("\n")) {
    const line = rawLine.trimEnd();
    const title = headingTitle(line);

    if (title) {
      if (DROP_SECTION.test(title)) {
        keeping = false;
        continue;
      }
      if (KEEP_SECTION.test(title)) {
        keeping = true;
        sawKeptHeading = true;
        output.push(`### ${title}`);
        continue;
      }
      keeping = !sawKeptHeading;
      if (keeping) {
        output.push(line);
      }
      continue;
    }

    if (!keeping && sawKeptHeading) {
      continue;
    }
    if (ARTIFACT_LINE.test(line) || COMMAND_LINE.test(line)) {
      continue;
    }
    if (keeping || !sawKeptHeading) {
      output.push(line);
    }
  }

  return output
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}
```

- [ ] **Step 4: Sanitize updater and fallback notes before caching**

In `src/lib/query/update-check.ts`:

```ts
import { sanitizeReleaseNotesForApp } from "@/lib/updates/sanitize-release-notes";
```

Before cache/return:

```ts
const notes = sanitizeReleaseNotesForApp(info.notes);
const sanitizedInfo = { ...info, notes: notes || null };
if (sanitizedInfo.notes && sanitizedInfo.version) {
  cacheReleaseNotes({
    version: sanitizedInfo.version,
    notes: sanitizedInfo.notes,
    channel: sanitizedInfo.channel,
    cachedAt: new Date().toISOString(),
  });
}
return sanitizedInfo;
```

In `src/lib/updates/current-release-notes.ts`, sanitize `release.body` before cache:

```ts
const notes = sanitizeReleaseNotesForApp(release.body);
```

In `src/lib/updates/release-notes-cache.ts`, sanitize legacy reads:

```ts
const parsedEntry = parsed.data;
return {
  ...parsedEntry,
  notes: sanitizeReleaseNotesForApp(parsedEntry.notes),
  source: parsedEntry.source ?? "updater",
};
```

Extend the cache schema:

```ts
const ReleaseNotesCacheSchema = z.object({
  version: z.string(),
  notes: z.string(),
  channel: z.string(),
  cachedAt: z.string(),
  source: z.enum(["updater", "installed-release"]).default("updater"),
});
```

In `src/lib/query/update-check.ts`, cache manifest notes with `source: "updater"`.

In `src/lib/updates/current-release-notes.ts`, cache GitHub fallback notes with `source: "installed-release"` because this path fetches the installed `appVersion()` tag unless a manifest has already supplied a newer version.

- [ ] **Step 5: Update update-note tests**

In `src/lib/query/update-check.test.ts`, change manifest notes to include install text and assert sanitized cache:

```ts
notes: "### What's changed\n- Current release only\n\n### Install stable\n- macOS: `curl example`",
```

Expected cache call:

```ts
notes: "### What's changed\n- Current release only",
```

In `src/lib/updates/current-release-notes.test.ts`, mock `fetch` and `appVersion` if not already available, and assert fallback cache receives sanitized notes for stable `v0.2.0`.

In `src/lib/updates/release-notes-cache.test.ts`, add:

```ts
it("defaults legacy cached notes to updater source", () => {
  localStorage.setItem(
    RELEASE_NOTES_CACHE_KEY,
    JSON.stringify({
      version: "0.2.0",
      notes: "### What's changed\n- Fix tray",
      channel: "stable",
      cachedAt: "2026-06-06T12:34:56.000Z",
    }),
  );

  expect(readCachedReleaseNotes()?.source).toBe("updater");
});
```

- [ ] **Step 6: Wire fallback note source into UI copy**

In `src/components/settings/settings-update-section.tsx`, keep using `updateInfo?.version` for available updates. When notes come only from `cachedNotes?.source === "installed-release"`, pass a description such as `Installed version notes` into `ReleaseNotesDialog` rather than presenting it as update notes.

In `src/components/updates/update-page.tsx`, compute:

```ts
const notesSource = updateInfo?.notes ? "updater" : (cachedNotes?.source ?? "updater");
const notesDescription =
  notesSource === "installed-release"
    ? "Release notes for the installed version."
    : "Release notes from the latest update check.";
```

Pass `notesDescription` to `UpdatePageContent` and `ReleaseNotesDialog`.

- [ ] **Step 7: Ensure notes views always scroll overflow**

In `src/components/updates/release-notes-dialog.tsx`, remove `hasScrollableNotes` and always wrap body content in:

```tsx
<ScrollFadeRegion
  orientation="vertical"
  controls="none"
  className="min-h-0 flex-1"
  scrollClassName="px-4 py-3"
>
  <PatchNotesSections sections={sections} />
  {sections.length === 0 ? (
    <p className="text-muted-foreground text-xs leading-relaxed">
      {isChecking
        ? "Checking for updates..."
        : "No release notes cached yet. Check for updates to fetch the latest notes."}
    </p>
  ) : null}
</ScrollFadeRegion>
```

In `src/components/updates/update-page-content.tsx`, use the same unconditional scrollport for notes body.

- [ ] **Step 8: Run focused tests**

```bash
pnpm test src/lib/updates/sanitize-release-notes.test.ts src/lib/query/update-check.test.ts src/lib/updates/current-release-notes.test.ts src/lib/updates/release-notes-cache.test.ts
```

Expected: pass.

- [ ] **Step 9: Commit**

```bash
git add src/lib/updates/sanitize-release-notes.ts src/lib/updates/sanitize-release-notes.test.ts src/lib/query/update-check.ts src/lib/query/update-check.test.ts src/lib/updates/current-release-notes.ts src/lib/updates/current-release-notes.test.ts src/lib/updates/release-notes-cache.ts src/lib/updates/release-notes-cache.test.ts src/components/updates/release-notes-dialog.tsx src/components/updates/update-page-content.tsx
git commit -m "fix(updates): sanitize release notes"
```

---

### Task 6: Updater Feed Builder and Tauri Config

**Files:**

- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/updater/mod.rs`
- Create: `scripts/release/build-updater-feed.mjs`
- Create: `scripts/release/build-updater-feed.test.mjs`
- Create: `scripts/release/validate-updater-feed.mjs`
- Modify: `package.json`

- [ ] **Step 1: Write failing feed-builder tests**

Create `scripts/release/build-updater-feed.test.mjs`:

```js
import { describe, expect, it } from "vitest";

import {
  buildUpdaterFeedEntries,
  endpointToPlatformKey,
  supportedRecoveryVersions,
} from "./build-updater-feed.mjs";

describe("updater feed builder", () => {
  it("maps endpoint target and arch to tauri platform keys", () => {
    expect(endpointToPlatformKey("darwin", "aarch64")).toBe("darwin-aarch64");
    expect(endpointToPlatformKey("darwin", "x86_64")).toBe("darwin-x86_64");
    expect(endpointToPlatformKey("linux", "x86_64")).toBe("linux-x86_64");
    expect(endpointToPlatformKey("windows", "x86_64")).toBe("windows-x86_64");
  });

  it("backfills minimum recovery versions", () => {
    expect(supportedRecoveryVersions(["0.2.1"])).toEqual(["0.1.7", "0.2.0", "0.2.1"]);
  });

  it("creates stable and unstable entries for every platform/version pair", () => {
    const entries = buildUpdaterFeedEntries({
      versions: ["0.1.7", "0.2.0"],
      channels: ["stable", "unstable"],
      latestVersion: "0.2.1",
      notes: "### What's changed\n- Fix updater",
      pubDate: "2026-06-06T12:34:56.000Z",
      artifacts: {
        "darwin-aarch64": {
          url: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1/Mochi_aarch64.app.tar.gz",
          signature: "sig-a",
        },
        "darwin-x86_64": {
          url: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1/Mochi_x64.app.tar.gz",
          signature: "sig-m",
        },
        "linux-x86_64": {
          url: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1/Mochi_0.2.1_amd64.AppImage.tar.gz",
          signature: "sig-l",
        },
        "windows-x86_64": {
          url: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1/Mochi_0.2.1_x64-setup.nsis.zip",
          signature: "sig-w",
        },
      },
    });

    expect(entries).toContainEqual(
      expect.objectContaining({
        path: "updates/darwin/aarch64/0.1.7/stable.json",
      }),
    );
    expect(entries).toContainEqual(
      expect.objectContaining({
        path: "updates/windows/x86_64/0.2.0/unstable.json",
      }),
    );
  });
});
```

- [ ] **Step 2: Run feed-builder tests and verify failure**

```bash
pnpm vitest run scripts/release/build-updater-feed.test.mjs
```

Expected: fails because the script does not exist.

- [ ] **Step 3: Strengthen Rust updater endpoint tests**

In `src-tauri/src/updater/mod.rs`, replace the broad endpoint assertions with exact URL expectations:

```rust
#[test]
fn update_endpoint_builds_exact_stable_feed_url() {
    let endpoint = update_endpoint_for_channel("stable").expect("stable endpoint");
    assert_eq!(
        endpoint.as_str(),
        "https://mochi-app.github.io/mochi/updates/%7B%7Btarget%7D%7D/%7B%7Barch%7D%7D/%7B%7Bcurrent_version%7D%7D/stable.json"
    );
}

#[test]
fn update_endpoint_builds_exact_unstable_feed_url() {
    let endpoint = update_endpoint_for_channel("unstable").expect("unstable endpoint");
    assert_eq!(
        endpoint.as_str(),
        "https://mochi-app.github.io/mochi/updates/%7B%7Btarget%7D%7D/%7B%7Barch%7D%7D/%7B%7Bcurrent_version%7D%7D/unstable.json"
    );
}

#[test]
fn update_endpoint_rejects_unknown_channel() {
    let error = update_endpoint_for_channel("beta").expect_err("beta rejected");
    assert!(error.contains("unsupported update channel: beta"));
}
```

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml update_endpoint
```

Expected: pass after the exact assertions match the current endpoint contract.

- [ ] **Step 4: Configure Tauri updater artifacts**

In `src-tauri/tauri.conf.json`, add:

```json
"bundle": {
  "active": true,
  "createUpdaterArtifacts": true,
  "targets": ["app", "dmg", "msi", "nsis", "appimage", "deb", "rpm"],
  "icon": [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.icns",
    "icons/icon.ico"
  ]
}
```

Keep updater `pubkey` as a placeholder in repo, but the workflows must replace it before building and fail if replacement is missing.

- [ ] **Step 5: Implement feed builder exports**

Create `scripts/release/build-updater-feed.mjs` with pure exports:

```js
export const ENDPOINTS = [
  ["darwin", "aarch64"],
  ["darwin", "x86_64"],
  ["linux", "x86_64"],
  ["windows", "x86_64"],
];

export function endpointToPlatformKey(target, arch) {
  const key = `${target}-${arch}`;
  if (key === "darwin-aarch64") return key;
  if (key === "darwin-x86_64") return key;
  if (key === "linux-x86_64") return key;
  if (key === "windows-x86_64") return key;
  throw new Error(`unsupported updater endpoint: ${target}/${arch}`);
}

export function supportedRecoveryVersions(extraVersions = []) {
  return Array.from(new Set(["0.1.7", "0.2.0", ...extraVersions])).sort((a, b) =>
    a.localeCompare(b, undefined, { numeric: true }),
  );
}

export function buildUpdaterFeedEntries({
  versions,
  channels,
  latestVersion,
  notes,
  pubDate,
  artifacts,
}) {
  const entries = [];
  for (const version of versions) {
    for (const channel of channels) {
      for (const [target, arch] of ENDPOINTS) {
        const platformKey = endpointToPlatformKey(target, arch);
        const artifact = artifacts[platformKey];
        if (!artifact) {
          throw new Error(`missing updater artifact for ${platformKey}`);
        }
        entries.push({
          path: `updates/${target}/${arch}/${version}/${channel}.json`,
          json: {
            version: latestVersion,
            notes,
            pub_date: pubDate,
            platforms: {
              [platformKey]: artifact,
            },
          },
        });
      }
    }
  }
  return entries;
}
```

Add CLI code that reads a JSON manifest path and output dir from args, writes every entry, and exits nonzero on missing artifacts.

- [ ] **Step 6: Implement validation helper**

Create `scripts/release/validate-updater-feed.mjs`:

```js
import { readFile } from "node:fs/promises";

export async function validateFeedFile(path) {
  const parsed = JSON.parse(await readFile(path, "utf8"));
  if (!parsed.version || !parsed.pub_date || !parsed.platforms) {
    throw new Error(`invalid updater feed: ${path}`);
  }
  for (const [platform, artifact] of Object.entries(parsed.platforms)) {
    if (!artifact?.url || !artifact?.signature) {
      throw new Error(`invalid updater artifact for ${platform} in ${path}`);
    }
  }
  return true;
}
```

Add CLI support for validating files passed as arguments.

- [ ] **Step 7: Add package script**

In `package.json`:

```json
"test:release": "vitest run scripts/release/*.test.mjs"
```

- [ ] **Step 8: Run release script tests**

```bash
pnpm vitest run scripts/release/build-updater-feed.test.mjs
cargo test --manifest-path src-tauri/Cargo.toml update_endpoint
pnpm format:check
```

Expected: pass.

- [ ] **Step 9: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/src/updater/mod.rs scripts/release/build-updater-feed.mjs scripts/release/build-updater-feed.test.mjs scripts/release/validate-updater-feed.mjs package.json
git commit -m "ci(updater): add feed builder"
```

---

### Task 7: Release Workflow Updater Publication

**Files:**

- Modify: `.github/workflows/release-stable.yml`
- Modify: `.github/workflows/release-unstable.yml`
- Modify: `docs/releasing.md`
- Create: `scripts/release/collect-updater-artifacts.mjs`
- Create: `scripts/release/collect-updater-artifacts.test.mjs`

- [ ] **Step 1: Add workflow tests by static assertions**

Create or extend a workflow source test, for example `scripts/release/workflow-updater.test.mjs`:

```js
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

for (const workflow of [
  ".github/workflows/release-stable.yml",
  ".github/workflows/release-unstable.yml",
]) {
  describe(workflow, () => {
    const source = readFileSync(workflow, "utf8");

    it("requires updater signing secrets before building", () => {
      expect(source).toContain("TAURI_SIGNING_PRIVATE_KEY");
      expect(source).toContain("TAURI_SIGNING_PRIVATE_KEY_PASSWORD");
      expect(source).toContain("MOCHI_UPDATER_PUBLIC_KEY");
    });

    it("builds and validates updater feeds", () => {
      expect(source).toContain("scripts/release/collect-updater-artifacts.mjs");
      expect(source).toContain("scripts/release/build-updater-feed.mjs");
      expect(source).toContain("scripts/release/validate-updater-feed.mjs");
      expect(source).toContain("https://mochi-app.github.io/mochi/updates");
      expect(source).toContain("curl --fail");
    });
  });
}
```

- [ ] **Step 2: Run workflow static test and verify failure**

```bash
pnpm vitest run scripts/release/workflow-updater.test.mjs
```

Expected: fails until workflows include the artifact collection, feed build, feed validation, and public key secret.

- [ ] **Step 3: Add signing/public-key guard to workflows**

In both release workflows before `tauri-action`, add:

```yaml
- name: Verify updater signing configuration
  shell: bash
  env:
    TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
    MOCHI_UPDATER_PUBLIC_KEY: ${{ secrets.MOCHI_UPDATER_PUBLIC_KEY }}
  run: |
    test -n "${TAURI_SIGNING_PRIVATE_KEY}" || { echo "missing TAURI_SIGNING_PRIVATE_KEY"; exit 1; }
    test -n "${TAURI_SIGNING_PRIVATE_KEY_PASSWORD}" || { echo "missing TAURI_SIGNING_PRIVATE_KEY_PASSWORD"; exit 1; }
    test -n "${MOCHI_UPDATER_PUBLIC_KEY}" || { echo "missing MOCHI_UPDATER_PUBLIC_KEY"; exit 1; }
    node -e '
      const fs = require("fs");
      const path = "src-tauri/tauri.conf.json";
      const pubkey = process.env.MOCHI_UPDATER_PUBLIC_KEY;
      const text = fs.readFileSync(path, "utf8").replace("MOCHI_UPDATER_PUBLIC_KEY_REPLACED_BY_CI", pubkey);
      if (text.includes("MOCHI_UPDATER_PUBLIC_KEY_REPLACED_BY_CI")) process.exit(1);
      fs.writeFileSync(path, text);
    '
```

- [ ] **Step 4: Build feed artifact manifest**

Create `scripts/release/collect-updater-artifacts.test.mjs`:

```js
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { describe, expect, it } from "vitest";

import { collectUpdaterArtifacts } from "./collect-updater-artifacts.mjs";

async function writeArtifact(root, relativePath, signature) {
  const path = join(root, relativePath);
  await mkdir(join(path, ".."), { recursive: true });
  await writeFile(path, "artifact");
  await writeFile(`${path}.sig`, signature);
}

describe("collectUpdaterArtifacts", () => {
  it("derives version, pubDate, release URLs, signatures, and stable channel", async () => {
    const root = await mkdtemp(join(tmpdir(), "mochi-updater-artifacts-"));
    await writeArtifact(root, "macos/Mochi_aarch64.app.tar.gz", "sig-darwin-arm");
    await writeArtifact(root, "macos/Mochi_x64.app.tar.gz", "sig-darwin-x64");
    await writeArtifact(root, "linux/Mochi_0.2.1_amd64.AppImage.tar.gz", "sig-linux");
    await writeArtifact(root, "windows/Mochi_0.2.1_x64-setup.nsis.zip", "sig-windows");

    const manifestPath = join(root, "updater-feed.json");
    const manifest = await collectUpdaterArtifacts({
      artifactRoot: root,
      channel: "stable",
      tagName: "v0.2.1",
      releaseBaseUrl: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1",
      releaseNotesPath: join(root, "missing-notes.md"),
      outputPath: manifestPath,
      pubDate: "2026-06-06T12:34:56.000Z",
    });

    expect(manifest.latestVersion).toBe("0.2.1");
    expect(manifest.channels).toEqual(["stable"]);
    expect(manifest.pubDate).toBe("2026-06-06T12:34:56.000Z");
    expect(manifest.artifacts["darwin-aarch64"].signature).toBe("sig-darwin-arm");
    expect(manifest.artifacts["linux-x86_64"].url).toBe(
      "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1/Mochi_0.2.1_amd64.AppImage.tar.gz",
    );
    expect(JSON.parse(await readFile(manifestPath, "utf8"))).toEqual(manifest);
  });

  it("derives an unstable version that is newer than the recovery versions", async () => {
    const root = await mkdtemp(join(tmpdir(), "mochi-updater-artifacts-"));
    await writeArtifact(root, "macos/Mochi_aarch64.app.tar.gz", "sig-darwin-arm");
    await writeArtifact(root, "macos/Mochi_x64.app.tar.gz", "sig-darwin-x64");
    await writeArtifact(root, "linux/Mochi_0.2.1_amd64.AppImage.tar.gz", "sig-linux");
    await writeArtifact(root, "windows/Mochi_0.2.1_x64-setup.nsis.zip", "sig-windows");

    const manifest = await collectUpdaterArtifacts({
      artifactRoot: root,
      channel: "unstable",
      tagName: "unstable-20260606.123456",
      unstableBaseVersion: "0.2.1",
      releaseBaseUrl:
        "https://github.com/BrainerVirus/mochi/releases/download/unstable-20260606.123456",
      releaseNotesPath: join(root, "missing-notes.md"),
      outputPath: join(root, "updater-feed.json"),
      pubDate: "2026-06-06T12:34:56.000Z",
    });

    expect(manifest.latestVersion).toBe("0.2.1-unstable.20260606.123456");
    expect(manifest.versions).toContain("0.1.7");
    expect(manifest.versions).toContain("0.2.0");
  });

  it("fails when an updater signature is missing", async () => {
    const root = await mkdtemp(join(tmpdir(), "mochi-updater-artifacts-"));
    await mkdir(join(root, "linux"), { recursive: true });
    await writeFile(join(root, "linux/Mochi_0.2.1_amd64.AppImage.tar.gz"), "artifact");

    await expect(
      collectUpdaterArtifacts({
        artifactRoot: root,
        channel: "unstable",
        tagName: "unstable-20260606.123456",
        unstableBaseVersion: "0.2.1",
        releaseBaseUrl:
          "https://github.com/BrainerVirus/mochi/releases/download/unstable-20260606.123456",
        releaseNotesPath: join(root, "missing-notes.md"),
        outputPath: join(root, "updater-feed.json"),
        pubDate: "2026-06-06T12:34:56.000Z",
      }),
    ).rejects.toThrow("missing updater artifact for darwin-aarch64");
  });
});
```

Run the new test and verify failure:

```bash
pnpm vitest run scripts/release/collect-updater-artifacts.test.mjs
```

Expected: fails because `scripts/release/collect-updater-artifacts.mjs` does not exist.

- [ ] **Step 5: Implement artifact discovery helper**

Create `scripts/release/collect-updater-artifacts.mjs`:

```js
import { readdir, readFile, writeFile } from "node:fs/promises";
import { basename, join } from "node:path";

const REQUIRED_ARTIFACTS = {
  "darwin-aarch64": [/aarch64\.app\.tar\.gz$/],
  "darwin-x86_64": [/(x64|x86_64)\.app\.tar\.gz$/],
  "linux-x86_64": [/amd64\.AppImage\.tar\.gz$/, /x86_64\.AppImage\.tar\.gz$/],
  "windows-x86_64": [/(x64|x86_64).*\.nsis\.zip$/],
};

async function listFiles(root) {
  const entries = await readdir(root, { recursive: true, withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile())
    .map((entry) => join(entry.parentPath ?? root, entry.name));
}

function versionFromTag(tagName, unstableBaseVersion) {
  const stable = /^v(?<version>\d+\.\d+\.\d+)$/.exec(tagName);
  if (stable?.groups?.version) return stable.groups.version;
  const unstable = /^unstable-(?<version>\d{8}\.\d{6})$/.exec(tagName);
  if (unstable?.groups?.version) {
    if (!unstableBaseVersion || !/^\d+\.\d+\.\d+$/.test(unstableBaseVersion)) {
      throw new Error("unstable releases require --unstableBaseVersion=X.Y.Z");
    }
    return `${unstableBaseVersion}-unstable.${unstable.groups.version}`;
  }
  throw new Error(`unsupported release tag for updater feed: ${tagName}`);
}

async function notesFromPath(path) {
  try {
    return await readFile(path, "utf8");
  } catch {
    return "### What's changed\n- See the GitHub release notes for this version.";
  }
}

export async function collectUpdaterArtifacts({
  artifactRoot,
  channel,
  tagName,
  unstableBaseVersion,
  releaseBaseUrl,
  releaseNotesPath,
  outputPath,
  pubDate = new Date().toISOString(),
}) {
  if (channel !== "stable" && channel !== "unstable") {
    throw new Error(`unsupported updater channel: ${channel}`);
  }
  const latestVersion = versionFromTag(tagName, unstableBaseVersion);

  const files = await listFiles(artifactRoot);
  const artifacts = {};
  for (const [platform, patterns] of Object.entries(REQUIRED_ARTIFACTS)) {
    const artifactPath = files.find(
      (file) => !file.endsWith(".sig") && patterns.some((pattern) => pattern.test(basename(file))),
    );
    if (!artifactPath) throw new Error(`missing updater artifact for ${platform}`);

    const signaturePath = `${artifactPath}.sig`;
    if (!files.includes(signaturePath)) {
      throw new Error(`missing updater signature for ${platform}: ${signaturePath}`);
    }

    artifacts[platform] = {
      url: `${releaseBaseUrl}/${basename(artifactPath)}`,
      signature: (await readFile(signaturePath, "utf8")).trim(),
    };
  }

  const manifest = {
    latestVersion,
    channels: [channel],
    versions: ["0.1.7", "0.2.0", latestVersion],
    notes: await notesFromPath(releaseNotesPath),
    pubDate,
    artifacts,
  };

  await writeFile(outputPath, `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const args = Object.fromEntries(
    process.argv.slice(2).map((arg) => {
      const [key, value] = arg.split("=");
      return [key.replace(/^--/, ""), value];
    }),
  );
  await collectUpdaterArtifacts({
    artifactRoot: args.artifactRoot,
    channel: args.channel,
    tagName: args.tagName,
    unstableBaseVersion: args.unstableBaseVersion,
    releaseBaseUrl: args.releaseBaseUrl,
    releaseNotesPath: args.releaseNotesPath,
    outputPath: args.outputPath,
    pubDate: args.pubDate,
  });
}
```

Run:

```bash
pnpm vitest run scripts/release/collect-updater-artifacts.test.mjs
```

Expected: pass.

- [ ] **Step 6: Build feed artifact manifest in workflows**

After each `tauri-action` build, upload the complete Tauri bundle directory as a workflow artifact so the feed job can inspect the real files:

```yaml
- name: Upload updater bundle artifacts
  uses: actions/upload-artifact@v4
  with:
    name: updater-bundle-${{ matrix.platform }}-${{ matrix.arch }}
    path: src-tauri/target/**/release/bundle/**
    if-no-files-found: error
```

In the feed publication job, download all uploaded updater bundles and collect the manifest:

```yaml
- name: Download updater bundle artifacts
  uses: actions/download-artifact@v4
  with:
    pattern: updater-bundle-*
    path: updater-bundles
    merge-multiple: true

- name: Build updater artifact manifest
  shell: bash
  env:
    CHANNEL: stable
    CHANNELS: stable,unstable
    TAG_NAME: ${{ github.ref_name }}
    RELEASE_BASE_URL: https://github.com/BrainerVirus/mochi/releases/download/${{ github.ref_name }}
  run: |
    node scripts/release/collect-updater-artifacts.mjs \
      --artifactRoot=updater-bundles \
      --channel="${CHANNEL}" \
      --tagName="${TAG_NAME}" \
      --releaseBaseUrl="${RELEASE_BASE_URL}" \
      --releaseNotesPath=release-notes.md \
      --outputPath=updater-feed.json \
      --pubDate="$(date -u +%Y-%m-%dT%H:%M:%S.000Z)"
```

For `.github/workflows/release-stable.yml`, keep `CHANNEL: stable`, set `CHANNELS: stable,unstable`, and fail the job unless `github.ref_name` matches `v[0-9]+.[0-9]+.[0-9]+`. Publishing both channel files from the stable recovery release guarantees that installed apps on the unstable channel do not keep receiving 404s before the next unstable build exists.

For `.github/workflows/release-unstable.yml`, set `CHANNEL: unstable`, set `CHANNELS: unstable`, pass `--unstableBaseVersion="${MOCHI_UNSTABLE_BASE_VERSION}"`, and use the unstable tag generated by that workflow. Add a guard that `MOCHI_UNSTABLE_BASE_VERSION` is a semver version greater than the minimum recovery version `0.2.0`; for the first repair release use `0.2.1`.

Before running `build-updater-feed.mjs`, rewrite the manifest `channels` field from the `CHANNELS` env var:

```bash
node -e '
  const fs = require("fs");
  const manifest = JSON.parse(fs.readFileSync("updater-feed.json", "utf8"));
  manifest.channels = process.env.CHANNELS.split(",").map((value) => value.trim()).filter(Boolean);
  fs.writeFileSync("updater-feed.json", `${JSON.stringify(manifest, null, 2)}\n`);
'
```

- [ ] **Step 7: Publish GitHub Pages feed**

Add a release job after all matrix builds that:

1. Downloads build artifacts.
2. Runs `node scripts/release/build-updater-feed.mjs --manifest updater-feed.json --out public`.
3. Runs:

```bash
node scripts/release/validate-updater-feed.mjs \
  public/updates/darwin/aarch64/0.1.7/stable.json \
  public/updates/darwin/aarch64/0.1.7/unstable.json \
  public/updates/darwin/x86_64/0.1.7/stable.json \
  public/updates/darwin/x86_64/0.1.7/unstable.json \
  public/updates/linux/x86_64/0.1.7/stable.json \
  public/updates/linux/x86_64/0.1.7/unstable.json \
  public/updates/windows/x86_64/0.1.7/stable.json \
  public/updates/windows/x86_64/0.1.7/unstable.json
```

4. Publishes `public/updates/**` to the GitHub Pages branch/repository path that serves `https://mochi-app.github.io/mochi/updates/{target}/{arch}/{current_version}/{channel}.json`.

Use explicit workflow permissions:

```yaml
permissions:
  contents: write
  pages: write
  id-token: write
```

If Pages publication needs a token or repository not currently configured, fail with a clear message and document the needed secret in `docs/releasing.md`.

After publishing Pages, add a post-publish HTTP validation step that waits for Pages propagation and verifies representative recovery endpoints by URL:

```bash
for path in \
  updates/darwin/aarch64/0.1.7/stable.json \
  updates/darwin/aarch64/0.1.7/unstable.json \
  updates/darwin/x86_64/0.2.0/stable.json \
  updates/linux/x86_64/0.2.0/unstable.json \
  updates/windows/x86_64/0.1.7/stable.json \
  updates/windows/x86_64/0.1.7/unstable.json
do
  url="https://mochi-app.github.io/mochi/${path}"
  curl --fail --silent --show-error --location "${url}" --output feed.json
  node scripts/release/validate-updater-feed.mjs feed.json
done
```

- [ ] **Step 8: Document release requirements**

In `docs/releasing.md`, add:

```md
## Updater Feed

Release workflows must generate signed Tauri updater artifacts and publish versioned feeds under `https://mochi-app.github.io/mochi/updates/{target}/{arch}/{current_version}/{channel}.json`.

Required secrets:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- `MOCHI_UPDATER_PUBLIC_KEY`
- GitHub Pages publication token if `GITHUB_TOKEN` cannot write the Pages source branch.

The feed is backfilled for supported installed versions, currently `0.1.7` and `0.2.0`, so older installed apps can recover through in-app update. Every supported recovery version must have both `stable.json` and `unstable.json` for macOS arm64/x64, Linux x64, and Windows x64.

The first stable repair release publishes both `stable.json` and `unstable.json` from tags like `v0.2.1` so installed unstable-channel apps recover from missing feeds. Later unstable releases publish `unstable.json` from tags like `unstable-20260606.123456` using a feed version such as `0.2.1-unstable.20260606.123456`, where `0.2.1` is the explicit `MOCHI_UNSTABLE_BASE_VERSION`. Workflows must fail when any required updater bundle or `.sig` file is missing.
```

- [ ] **Step 9: Run workflow source tests**

```bash
pnpm vitest run scripts/release/workflow-updater.test.mjs scripts/release/build-updater-feed.test.mjs scripts/release/collect-updater-artifacts.test.mjs
```

Expected: pass.

- [ ] **Step 10: Commit**

```bash
git add .github/workflows/release-stable.yml .github/workflows/release-unstable.yml docs/releasing.md scripts/release/workflow-updater.test.mjs scripts/release/collect-updater-artifacts.mjs scripts/release/collect-updater-artifacts.test.mjs
git commit -m "ci(updater): publish versioned feeds"
```

---

### Task 8: Final Verification and Manual QA Checklist

**Files:**

- Modify: `docs/linux.md`
- Modify: `docs/releasing.md`
- Modify: `.github/PULL_REQUEST_TEMPLATE.md` if the template lacks Linux/updater manual QA checkboxes.

- [ ] **Step 1: Add QA checklist docs**

In `docs/linux.md`, add a troubleshooting/QA subsection:

```md
### Tray and Widget QA

On Ubuntu, verify:

- Native tray menu does not contain `Show usage`.
- `Open widget` is the first action.
- Stable/unstable channel submenu shows the current channel checked.
- Selecting a provider tab keeps that provider percentage in the tray after refresh interval elapses.
- Mouse wheel and trackpad scrolling work in widget, settings, update page, and What's New dialog.
- Settings and widget native minimize/maximize/close controls work.
```

In `docs/releasing.md`, add endpoint validation examples:

```bash
curl -fsS https://mochi-app.github.io/mochi/updates/darwin/aarch64/0.1.7/stable.json
curl -fsS https://mochi-app.github.io/mochi/updates/linux/x86_64/0.1.7/stable.json
curl -fsS https://mochi-app.github.io/mochi/updates/windows/x86_64/0.1.7/stable.json
```

- [ ] **Step 2: Run full frontend gate**

```bash
pnpm lint
pnpm format:check
pnpm test
pnpm build
```

Expected: all pass.

- [ ] **Step 3: Run full Rust gate**

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: all pass.

- [ ] **Step 4: Run local Tauri smoke where possible**

On the current development machine:

```bash
pnpm tauri dev
```

Verify:

- app starts,
- tray icon appears,
- settings opens,
- widget opens,
- update page opens.

If Linux is not the current machine, record that Ubuntu installed-package QA remains required before release.

- [ ] **Step 5: Commit documentation checklist**

```bash
git add docs/linux.md docs/releasing.md .github/PULL_REQUEST_TEMPLATE.md
git commit -m "docs: add linux updater qa"
```

---

## Final Integration

- [ ] **Step 1: Run all validation commands**

```bash
pnpm lint
pnpm format:check
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

- [ ] **Step 2: Review git diff**

```bash
git diff --stat main...HEAD
git diff main...HEAD -- src-tauri/src/tray/mod.rs src/hooks/use-tray-events.ts src/lib/stores/tray-ui-store.ts src-tauri/tauri.conf.json .github/workflows/release-stable.yml .github/workflows/release-unstable.yml
```

- [ ] **Step 3: Prepare PR notes from template**

Use `.github/PULL_REQUEST_TEMPLATE.md` and include:

- tray menu cleanup,
- provider percentage persistence,
- Linux scroll/widget/settings parity,
- release-note sanitization,
- updater feed generation/backfill,
- validation commands and any manual QA not run locally.
