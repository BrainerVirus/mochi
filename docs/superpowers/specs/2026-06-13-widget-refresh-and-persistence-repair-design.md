# Widget Refresh, Tab Persistence & Cold-Start Repair

**Date:** 2026-06-13
**Status:** Design approved, pending implementation

## Overview

Three related issues affect the Mochi widget window on Linux (and potentially other platforms): tray refresh doesn't update the widget, the widget opens at the wrong tab, and the widget shows stale data on open. Each has a distinct root cause.

| #   | Issue                                              | Root Cause                                                                                                                                                                           |
| --- | -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | Tray "Refresh usage" doesn't update widget         | Race condition: both windows call `refreshEnabledProviders()` simultaneously; `RefreshController` skips the second call, which refetches cached data before the first call completes |
| 2   | Widget opens at "overview" tab instead of selected | `localStorage` is per-window; tab preference stored in main window's localStorage is invisible to widget window                                                                      |
| 3   | Widget shows stale data on open                    | Cold-start refresh only triggers for `kind === "fetching"`, but cached snapshots are `"fresh"` or `"stale_error"` — neither triggers a refresh                                       |

---

## Issue 1: Rust-Orchestrated Refresh (Push Model)

### Problem

Both windows independently receive `tray-refresh` and race on `refreshEnabledProviders()`. The `RefreshController` (a `static OnceLock<Mutex<HashSet<ProviderId>>>`) prevents concurrent refreshes per-provider. The second caller is skipped, refetches cache before the first completes, and gets stale data.

### Solution

Move refresh orchestration to the Rust side. A single command runs the refresh to completion, then pushes results to all windows.

#### New Rust Commands

```rust
// src-tauri/src/status/mod.rs

#[derive(Serialize)]
struct RefreshCompletePayload {
    snapshots: Vec<UsageSnapshot>,
}

#[tauri::command]
pub async fn refresh_all_providers(
    app: AppHandle,
    store: State<'_, UsageStore>,
    settings_state: State<'_, SettingsState>,
) -> Result<(), String> {
    let settings = settings_state.current()?;
    refresh_enabled_snapshots(&store, &settings).await?;
    let snapshots = read_cached_snapshots(&store, &settings);
    let _ = app.emit("usage-refresh-complete", RefreshCompletePayload { snapshots });
    Ok(())
}

#[tauri::command]
pub async fn refresh_single_provider(
    app: AppHandle,
    store: State<'_, UsageStore>,
    settings_state: State<'_, SettingsState>,
    provider: String,
) -> Result<(), String> {
    let settings = settings_state.current()?;
    // Existing refresh_provider logic, but emit push event after
    let provider_id = ProviderId::parse(&provider).ok_or_else(|| "unknown provider".to_string())?;
    let ctx = FetchContext::from_settings(&settings);
    match fetch_provider_snapshot(provider_id, &ctx).await {
        Ok(Some(snapshot)) => { store.record_success(snapshot); }
        // ... existing error handling from refresh_provider ...
    }
    let snapshots = read_cached_snapshots(&store, &settings);
    let _ = app.emit("usage-refresh-complete", RefreshCompletePayload { snapshots });
    Ok(())
}
```

#### Tray Menu Handler Change

In `src-tauri/src/tray/mod.rs`, the `"refresh"` menu event:

```
Before:  app.emit("tray-refresh", ())
After:   // call refresh_all_providers command instead
         // This is done by emitting a new event that the main window picks up,
         // OR by calling the command directly from Rust.
```

Since the tray handler is on the Rust side, it calls the new command directly rather than emitting an event to the frontend:

```rust
"refresh" => {
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::status::refresh_all_providers_inner(&app_clone).await;
    });
}
```

Where `refresh_all_providers_inner` is the internal function (without Tauri state wrappers) that does the work.

#### Frontend Changes

**Remove** the `"tray-refresh"` listener from `useTrayEvents`.

**Add** a `"usage-refresh-complete"` listener that pushes data directly into TanStack Query cache:

```typescript
listen<RefreshCompletePayload>("usage-refresh-complete", (event) => {
  queryClient.setQueryData(queryKeys.usageSnapshots, event.payload.snapshots);
});
```

**Simplify widget refresh buttons:**

- Global refresh → calls `invoke("refresh_all_providers")`
- Per-provider refresh → calls `invoke("refresh_single_provider", { provider })`
- Both buttons no longer manage invalidation/refetch manually

**Remove:**

- `runTrayRefreshEventSequence` function
- `shouldRunProviderRefreshForTrayEvent` function
- Manual `invalidateQueries` + `refetch` logic from `useTrayPanelRefresh`
- `useTrayPanelRefresh` simplified to just a command-invocation bridge

#### Data Flow

```
Tray "Refresh usage" / Widget refresh button
        │
        ▼
  Rust: refresh_all_providers()
        │
        ├── refresh_enabled_snapshots() — single call, no race
        │   └── RefreshController locks per-provider (no contention)
        │
        └── emit("usage-refresh-complete", { snapshots })
               │
        ┌───────┴──────────┐
        ▼                  ▼
   main window        widget window
   setQueryData()     setQueryData()
   (fresh data)       (fresh data)
```

#### Files Changed

| File                                                                       | Change                                                                                                  |
| -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/status/mod.rs`                                              | Add `refresh_all_providers`, `refresh_single_provider` commands; add `RefreshCompletePayload`           |
| `src-tauri/src/tray/mod.rs`                                                | `"refresh"` handler → call refresh command directly                                                     |
| `src-tauri/src/lib.rs`                                                     | Register new commands                                                                                   |
| `src/features/tray/hooks/use-tray-events/use-tray-events.ts`               | Replace `"tray-refresh"` with `"usage-refresh-complete"` listener; remove `runTrayRefreshEventSequence` |
| `src/features/tray/hooks/use-tray-panel-refresh/use-tray-panel-refresh.ts` | Simplify to invoke Rust commands                                                                        |
| `src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.ts`     | Update refresh paths to use new commands                                                                |
| `src/lib/tauri/commands/commands.ts`                                       | Add `refreshAllProviders`, `refreshSingleProvider` bindings                                             |

---

## Issue 2: Selected Tab Persistence (No localStorage)

### Problem

Selected tab is stored in `localStorage` key `mochi-tray-selected-tab`. Tauri v2 webview windows have separate localStorage backends — the widget window reads from its own empty localStorage and defaults to `"overview"`.

### Solution

Move the selected tab to the shared `MochiSettings` (Rust-side `settings.json`). Inject it into the widget via an **initialization script** on first creation, and via a `"set-tab"` event on re-show. The initialization script runs before any frontend JS — truly synchronous, no flash.

#### Rust: `MochiSettings` new field

Add `selected_tab: Option<String>` to `MochiSettings`:

```rust
// src-tauri/src/settings/mod.rs
pub struct MochiSettings {
    // ...existing fields...
    #[serde(default)]
    pub selected_tab: Option<String>,  // None → "overview"
}
```

Default value is `None`, which the frontend interprets as `"overview"`.

#### Rust: Inject initial tab via `initialization_script` on widget creation

Tauri v2's `initialization_script` runs before any page JavaScript — the value is available at module-init time, before React renders:

```rust
// src-tauri/src/widget/commands.rs
fn build_widget_window(app: &AppHandle) -> Result<WebviewWindow, tauri::Error> {
    let selected_tab = app
        .try_state::<SettingsState>()
        .and_then(|state| state.current().ok())
        .and_then(|s| s.selected_tab.clone())
        .unwrap_or_default();

    let init_script = format!(
        "window.__MOCHI_SELECTED_TAB__ = '{}';",
        selected_tab
    );

    let window = tauri::WebviewWindowBuilder::new(app, WIDGET_LABEL, app_shell_url())
        .title("Mochi Widget")
        .initialization_script(&init_script)
        // ... rest unchanged ...
        .build()?;
    Ok(window)
}
```

#### Rust: Emit `"set-tab"` only for reused windows (re-show path)

The `"set-tab"` event must only be emitted when the widget window already existed. For first creation, the URL param handles initialization — the frontend hasn't loaded yet, so the event would be lost.

```rust
// src-tauri/src/widget/commands.rs — show_widget
pub fn show_widget(app: AppHandle) -> Result<(), String> {
    let already_existed = app.get_webview_window(WIDGET_LABEL).is_some();
    let window = ensure_widget_window(&app)?;
    let policy = crate::window_policy::active_decorated_window_policy();
    let creation = policy.creation_label();
    let initial_visibility = policy.initial_visibility_label();

    // NEW: emit current selected tab before showing (for re-show only)
    if already_existed {
        if let Some(state) = app.try_state::<SettingsState>() {
            if let Ok(settings) = state.current() {
                if let Some(tab) = settings.selected_tab.clone() {
                    let _ = app.emit_to(WIDGET_LABEL, "set-tab", tab);
                }
            }
        }
    }

    // ... rest of existing show_widget logic unchanged ...
}
```

#### Frontend: Zustand reads initial tab from `window.__MOCHI_SELECTED_TAB__`

The script runs before any React code — value is available at module-init time:

```typescript
// src/features/tray/lib/stores/tray-ui-store/tray-ui-store.ts
function readStoredTab(): TraySelectedTab {
  // NEW: synchronous init script value (set by Rust build_widget_window)
  const initialTab = (window as unknown as Record<string, unknown>).__MOCHI_SELECTED_TAB__;
  if (typeof initialTab === "string") {
    if (initialTab === "overview") return "overview";
    const parsed = ProviderIdSchema.safeParse(initialTab);
    if (parsed.success) return parsed.data;
  }

  return "overview";
}
```

#### Frontend: Listen for `"set-tab"` event

```typescript
// In useTrayEvents or a new useWidgetEvents hook
useEffect(() => {
  const unlisten = listen<string>("set-tab", (event) => {
    const tab = parseTrayTabChange(event.payload);
    useTrayUiStore.getState().setSelectedTab(tab);
  });
  return () => {
    void unlisten.then((fn) => fn());
  };
}, []);
```

#### Frontend: Persist tab to settings on change

```typescript
// useTrayPanelState.ts — handleTabChange
function handleTabChange(value: string) {
  const nextTab = parseTrayTabChange(value);
  setSelectedTab(nextTab);
  void syncTrayUsage(nextTab);

  // NEW: Save to shared settings
  if (settings) {
    void saveSettings({ ...settings, selected_tab: nextTab });
  }
}
```

#### Data Flow

```
┌─ First widget open ───────────────────────────┐
│ Rust: inject window.__MOCHI_SELECTED_TAB__     │
│   via initialization_script                    │
│ Frontend: readStoredTab() reads from global    │
│ → correct tab on frame 1 (synchronous)         │
└────────────────────────────────────────────────┘

┌─ Widget re-shown (was hidden) ────────────────┐
│ Rust: emit "set-tab" → "opencode"              │
│ Rust: window.show()                            │
│ Frontend: listener fires → Zustand updates     │
│ → correct tab before paint                     │
└────────────────────────────────────────────────┘

┌─ Tab change anywhere ─────────────────────────┐
│ User clicks tab → Zustand + syncTrayUsage()    │
│ Also → saveSettings({..., selected_tab: X})    │
│ Shared settings.json → consistent across opens  │
└────────────────────────────────────────────────┘
```

#### Removed

- `localStorage.getItem` / `setItem` calls from `tray-ui-store.ts`
- The `STORAGE_KEY` constant
- No SQLite `preferences` table needed — uses existing `MochiSettings` JSON

#### Files Changed

| File                                                                   | Change                                                                                              |
| ---------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `src-tauri/src/settings/mod.rs`                                        | Add `selected_tab: Option<String>` to `MochiSettings`                                               |
| `src-tauri/src/widget/commands.rs`                                     | Add `initialization_script` to `build_widget_window`; emit `"set-tab"` in `show_widget` for re-show |
| `src/features/tray/lib/stores/tray-ui-store/tray-ui-store.ts`          | Read `window.__MOCHI_SELECTED_TAB__` instead of localStorage                                        |
| `src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.ts` | Save `selected_tab` to settings on tab change                                                       |
| `src/features/tray/hooks/use-tray-events/use-tray-events.ts`           | Add `"set-tab"` listener                                                                            |
| `src/features/widget/components/widget-window/widget-window.tsx`       | (maybe) mount `"set-tab"` listener                                                                  |

---

## Issue 3: Expanded Cold-Start Refresh

### Problem

`shouldRefreshEnabledProvidersOnBoot` only triggers when `kind === "fetching"`. Cached snapshots are typically `"fresh"` (data exists) or `"stale_error"` (last fetch failed). Neither triggers a refresh, so the widget shows whatever data was last cached.

### Solution

Expand the trigger conditions to also handle `"stale_error"`, `"error"`, and add a **time-based trigger** for "fresh" data older than the configured refresh interval.

#### New Logic

```typescript
// src/hooks/use-cold-start-provider-refresh.ts

export function shouldRefreshEnabledProvidersOnBoot(
  settings: MochiSettings,
  states: ProviderUsageState[],
): boolean {
  if (settings.enabled_providers.length === 0) return false;

  const enabled = new Set(settings.enabled_providers);
  const refreshInterval = settings.refresh_interval_seconds; // default 300

  return states.some((state) => {
    if (!enabled.has(state.provider)) return false;

    // Always retry failed/error fetches
    if (state.kind === "fetching") return true;
    if (state.kind === "stale_error") return true;
    if (state.kind === "error") return true;

    // Time-based: refresh "fresh" data if it's older than the interval
    if (state.kind === "fresh" && state.updated_at) {
      const ageMs = Date.now() - new Date(state.updated_at).getTime();
      const thresholdMs = refreshInterval * 1000;
      return ageMs > thresholdMs;
    }

    return false;
  });
}
```

#### Behavior by State Kind

| State Kind                 | Before        | After                                               | Rationale                             |
| -------------------------- | ------------- | --------------------------------------------------- | ------------------------------------- |
| `fresh`                    | ❌ no refresh | ✅ refresh if older than `refresh_interval_seconds` | Stale cached data needs updating      |
| `fetching`                 | ✅ refresh    | ✅ refresh (unchanged)                              | Never fetched, needs to be fetched    |
| `stale_error`              | ❌ no refresh | ✅ refresh                                          | Last fetch failed, retry on open      |
| `error`                    | ❌ no refresh | ✅ refresh                                          | Error state, retry on open            |
| `missing_credentials`      | ❌ no refresh | ❌ no refresh                                       | Nothing to fetch, user must configure |
| `credentials_need_refresh` | ❌ no refresh | ❌ no refresh                                       | User must re-authenticate             |

#### Deduplication

The existing `didRefreshRef` ref prevents duplicate refreshes per-window session:

```typescript
const didRefreshRef = useRef(false);
useEffect(() => {
  if (didRefreshRef.current || ...) return;
  didRefreshRef.current = true;
  // run refresh
}, [queryClient, settings, states]);
```

With Issue 1's push model, even if both windows fire `refreshEnabledProviders()` simultaneously, the `RefreshController` deduplicates, and the push event updates both windows with the result.

#### No New Mounting Needed

`useColdStartProviderRefresh()` is already called in `TrayEventBridge`, which is mounted by `RootComponent` in all windows (main, widget, settings). The expanded logic automatically works everywhere.

#### Files Changed

| File                                                | Change                                                                                                |
| --------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `src/hooks/use-cold-start-provider-refresh.ts`      | Expand `shouldRefreshEnabledProvidersOnBoot` to handle `stale_error`, `error`, and time-based `fresh` |
| `src/hooks/use-cold-start-provider-refresh.test.ts` | Update tests for new conditions                                                                       |

---

## Acceptance Criteria

### Issue 1

- [ ] Tray "Refresh usage" updates widget content immediately (no manual re-refresh needed)
- [ ] Widget "global refresh" button works via Rust push model
- [ ] Widget "per-provider refresh" button works via Rust push model
- [ ] Both windows receive updated snapshots simultaneously
- [ ] Tray icon updates correctly after refresh
- [ ] Removed `"tray-refresh"` event emission from tray handler
- [ ] Removed `runTrayRefreshEventSequence` and related code

### Issue 2

- [ ] Widget opens at correct selected tab on first creation (URL param)
- [ ] Widget re-shows at correct selected tab (`"set-tab"` event)
- [ ] Tab change persists across app restarts (stored in `settings.json`)
- [ ] Tab change in tray panel is reflected on next widget open
- [ ] Tab change in widget is reflected in tray panel on next tray open
- [ ] No localStorage reads/writes for tab preference
- [ ] No flash of incorrect tab on widget open

### Issue 3

- [ ] Widget auto-refreshes stale providers (`kind === "stale_error"` or `"error"`) on open
- [ ] Widget auto-refreshes if cached data is older than `refresh_interval_seconds`
- [ ] Providers with `kind === "fetching"` still auto-refresh (unchanged)
- [ ] `missing_credentials` and `credentials_need_refresh` are not auto-refreshed
- [ ] No duplicate refreshes within same window session (`didRefreshRef` guard)

---

## Implementation Order

1. **Issue 1 first** — establishes the Rust push model that Issue 3's refresh flow will use
2. **Issue 3 second** — expands cold-start refresh triggers (no conflicts with Issue 1)
3. **Issue 2 third** — tab persistence (independent of refresh flow)
