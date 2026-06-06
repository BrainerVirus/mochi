# Linux Native Window Controls Root-Cause Spec

**Status:** Investigation spec, not an implementation plan.

**Problem:** On Ubuntu Wayland, native minimize/maximize/close controls on decorated Mochi windows render but do not hover or click until the user manually maximizes/restores by double-clicking the top-center titlebar area. After the affected window is closed and opened again, the broken hit regions return. The affected surfaces are the settings/about/update window and the widget window. Scrolling now works; this spec is about native controls and widget/settings visual parity.

**Scope:** Tauri v2 desktop window creation/show lifecycle, Linux window diagnostics, root document window attributes, and app-window CSS. Provider fetching, updater feeds, tray menu behavior, and scroll primitives are out of scope except where they affect the affected windows.

---

## Evidence From Current Reports

### Environment

From the 2026-06-06 diagnostics bundle:

- `platform: linux`
- `env.XDG_SESSION_TYPE: wayland`
- `env.WAYLAND_DISPLAY: wayland-0`
- `env.DISPLAY: :0`
- `env.GDK_BACKEND: (unset)`
- `env.WEBKIT_DISABLE_DMABUF_RENDERER: 1`
- `env.WEBKIT_DISABLE_COMPOSITING_MODE: 1`
- `env.MOCHI_ALLOW_WEBKIT_ACCELERATION: (unset)`

The installed build being tested is `mochi 0.2.1`.

### Behavior

User-observed behavior:

- Native controls render on settings/about/widget windows.
- Hover states do not react.
- Clicking minimize/maximize/close does not work.
- Double-clicking the middle/top titlebar area maximizes the window.
- Double-clicking again restores it.
- After this manual maximize/restore cycle, native controls begin hovering and clicking.
- Closing and reopening the same surface makes the bug return.

This is not consistent with missing decorations. It is consistent with stale or uninitialized titlebar hit-test regions that the window manager repairs during maximize/restore.

### Logs Before Earlier Fixes

Older logs showed repeated runtime decoration mutations:

```text
widget: linux_set_decorations -> ok
widget: linux_set_resizable -> ok
settings: linux_set_decorations -> ok
settings: linux_set_resizable -> ok
```

That led to the decoration-reset hypothesis.

### Logs After Decoration Mutation Was Removed

Latest logs show passive diagnostics only:

```text
[window.linux_controls] label=settings platform=linux source=rust-builder decorations=linux_builder_decorations ok=true error=None resizable=linux_builder_resizable ok=true error=None
[window] created label=settings url=tauri://localhost shell=index.html decorations=true transparent=false
[window.linux_controls] label=widget platform=linux source=tauri-config decorations=linux_builder_decorations ok=true error=None resizable=linux_builder_resizable ok=true error=None
[window.snapshot] widget url=tauri://localhost visible=false
[window.snapshot] main url=tauri://localhost visible=false
[window.snapshot] settings url=tauri://localhost visible=false
```

Then on open:

```text
widget: show -> ok
widget: unminimize -> ok
widget: set_focus -> ok
settings: show -> ok
settings: unminimize -> ok
settings: set_focus -> ok
```

The native-control bug still occurs after these logs. Therefore `set_decorations(true)` / `set_resizable(true)` after creation is **falsified as the complete root cause**. It may have been harmful, but removing it did not fix the real issue.

### Shared Remaining Fact

Both affected decorated windows are still created hidden:

- Settings/about/update: `src-tauri/src/tray/panel.rs` creates the `settings` webview at startup via `setup_app_windows()`, with `.visible(false)`.
- Widget: `src-tauri/tauri.conf.json` predeclares the `widget` window with `"visible": false`, and the Rust fallback in `src-tauri/src/widget/commands.rs` also uses `.visible(false)`.
- Latest diagnostics confirm both `settings` and `widget` snapshots are `visible=false` after creation.

The failure returns after close/reopen. The current widget path recreates hidden and then shows. The current settings path recreates or reuses a hidden dedicated app window and then shows. This aligns with the stale-hit-region symptom.

---

## Falsified Or Insufficient Attempts

These attempts must not be repeated as final fixes without new evidence:

1. **Calling `set_decorations(true)` and `set_resizable(true)` on Linux after creation**
   - Logs showed calls succeeded.
   - Controls still failed.
   - Latest build removed those calls and the bug still occurs.

2. **Changing close from hide to real close**
   - This is correct native behavior, but it did not address the hover/click dead zones.

3. **Removing widget always-on-top and max-size caps**
   - This was correct for normal maximize behavior, but did not address dead native-control hit regions.

4. **Widget shell visual cleanup**
   - Scrolling improved and the duplicate container was reduced, but native controls still fail.

No future implementation may claim success solely because these code paths are clean. The first-open hover/click behavior must be verified on Ubuntu Wayland.

---

## Primary Hypothesis To Test

**Hypothesis A:** The root cause is hidden pre-creation of decorated GTK/WebKit windows on Ubuntu Wayland. Tauri/Wry creates the native decorated frame while hidden, then Mochi mutates size/route/focus and shows it later. The titlebar is drawn, but its interactive hit regions are stale until the window manager recalculates the frame during maximize/restore.

Why this matches the data:

- All affected windows are decorated and hidden at creation.
- The tray popover is not relevant because it is intentionally undecorated.
- Native controls begin working only after a window-manager geometry state transition.
- Closing/reopening reintroduces the bad state.
- Other Tauri apps on the same machine do not show the issue; ordinary Tauri apps usually create their main decorated window visible as part of app startup instead of keeping multiple decorated windows hidden and later showing them.

This hypothesis is not yet proven. The implementation plan must include a diagnostic build that proves or disproves it.

---

## Secondary Hypotheses To Test Only If A Fails

### Hypothesis B: Size mutation while hidden or immediately before show corrupts titlebar hit regions

`open_app_window()` currently calls `set_size()` and `set_min_size()` before showing the app window. Widget height can also be controlled through `set_widget_height()`. If visible-on-create alone does not fix the bug, test whether size/min-size mutations must be moved into the builder or deferred until after the first map/configure event.

### Hypothesis C: The show sequence is wrong for GTK/Wayland

Current sequence:

```text
show -> unminimize -> set_focus
```

If visible-on-demand creation is insufficient, test:

```text
show -> set_focus
```

and:

```text
unminimize -> show -> set_focus
```

Do not ship a sequence change unless a diagnostic build proves it changes the first-open hover/click result.

### Hypothesis D: Frontend shell creates a webview overlay over native chrome

This is less likely because native double-click maximize works and controls recover after maximize/restore, but it must be ruled out if A-C fail. Check whether any app-region/drag-region CSS or webview bounds overlap the server-side/client-side titlebar region on Linux.

---

## Widget Background Mismatch

The widget does not get the same document-level shell attributes as settings:

- `RootComponent` sets `html[data-app-window]` only for the `settings` label.
- `RootComponent` sets `html[data-widget-window]` for the widget label.
- `src/styles/index.css` app-window html/body background rules target `html[data-app-window]`, but not `html[data-widget-window]`.
- The Linux opaque shell selector also omits `data-widget-window`.

The widget component uses the `.app-window` class, but the document/body/root shell rules do not fully match settings. This can make the widget show a different background around or behind the shared app-window surface.

**Constraint:** Widget must either be treated as an app-window shell at the root attribute level or every relevant app-window html/body/dialog/shell selector must include `html[data-widget-window]`. Partial component-only styling is not sufficient.

---

## Required Diagnostic Instrumentation

Before implementing a permanent fix, add diagnostics that distinguish window lifecycle states. The diagnostics must be available in `mochi diagnostics --bundle`.

For `settings`, `about`, `update`, and `widget`, record:

- window label
- logical requested route
- creation source: `startup-precreate`, `on-demand-builder`, or `tauri-config`
- whether the builder used `visible(true)` or `visible(false)`
- whether the window was created at startup or in response to a user action
- `decorations` requested at creation
- `resizable` requested at creation
- `transparent` requested at creation
- whether `show`, `unminimize`, `set_focus`, `set_size`, and `set_min_size` were called before the first visible event
- logical outer size and inner size immediately after creation, after show, after focus, and after manual resize/maximize if events expose this
- close behavior: `native-close-destroy`, `hide`, or `prevented`

Do not log only “ok”. The diagnostic must log sequencing, because sequencing is the suspected root cause.

---

## Diagnostic Build Matrix

Run these variants on Ubuntu Wayland using an installed package, not only `tauri dev`.

### Variant 1: Current Baseline

Use current behavior. Confirm the repro and capture diagnostics:

- settings first open
- about first open
- update first open
- widget first open
- close and reopen each
- double-click maximize/restore workaround

Expected baseline: controls fail until maximize/restore.

### Variant 2: On-Demand Visible Decorated Windows

For Linux only, do not pre-create settings/widget decorated windows at startup.

Settings/about/update:

- Remove `setup_app_windows()` pre-creation for Linux.
- Create the `settings` webview on first `open_app_window()` call.
- Builder must request the final route/size/min-size/decorations/resizable before show.
- Prefer creating visible at the user-action time instead of hidden startup creation.
- Do not call decoration/resizable setters after build.

Widget:

- Do not predeclare the widget as a startup hidden window on Linux.
- Create it on first `show_widget()`.
- Builder must request `decorations(true)`, `resizable(true)`, normal taskbar behavior, and the final initial size.
- Prefer creating visible at the user-action time instead of creating hidden and then showing.

Expected result if Hypothesis A is correct: controls hover and click on first open and after close/reopen without maximize/restore.

### Variant 3: Hidden On-Demand Control

Only if Variant 2 fixes the bug, run a control build that creates on-demand but still hidden before show. This isolates whether the root cause is startup precreation or hidden decorated creation in general.

Expected:

- If hidden on-demand fails, then `visible(false)` decorated creation is the root cause.
- If hidden on-demand passes, then startup precreation or frontend boot timing is the root cause.

### Variant 4: Size Mutation Isolation

Only if Variant 2 does not fix the bug:

- Keep on-demand creation.
- Remove `set_size()` and `set_min_size()` calls before first show.
- Use builder-only size/min-size.
- If route-specific about/update sizes are needed, create separate labels/windows for about/update in the diagnostic build instead of resizing a hidden/reused settings window.

Expected:

- If controls work, pre-show size mutation is the root cause.

---

## Implementation Constraints

The permanent fix must obey these constraints:

1. Do not call `set_decorations(true)` or `set_resizable(true)` after a decorated Linux window has been built.
2. Do not pre-create decorated Linux app/widget windows hidden at startup unless the diagnostic matrix proves it is safe.
3. Do not mutate size/min-size before first show unless the diagnostic matrix proves it is safe.
4. Do not use custom titlebars as a workaround while native controls are expected.
5. Do not ship a focus/unminimize reorder without Ubuntu evidence that it fixes first-open hover/click.
6. Keep macOS overlay titlebar behavior guarded by `#[cfg(target_os = "macos")]`.
7. Keep the tray popover undecorated, skip-taskbar, and always-on-top; this spec does not change tray popover behavior.
8. Widget and settings must share app-window background tokens and document/body shell rules.
9. Widget must remain scrollable with native wheel/trackpad scrolling.
10. Close/reopen must be tested for every affected surface.

---

## Required Tests

### Source Tests

Add or update tests that assert:

- Linux decorated windows are not pre-created hidden at startup.
- Linux decorated windows do not call post-build decoration/resizable setters.
- Widget config does not create a hidden startup Linux widget window if the diagnostic build proves on-demand creation is required.
- Widget root shell shares app-window document/body selectors with settings.
- App-window selectors include both `html[data-app-window]` and `html[data-widget-window]`, or widget deliberately receives `data-app-window` as well.

### Rust Tests

Add unit tests for any pure window-policy functions introduced, for example:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecoratedWindowCreationMode {
    StartupHidden,
    OnDemandVisible,
}
```

Test expected policy:

- Linux settings/about/update/widget use `OnDemandVisible`.
- macOS may keep the existing overlay/titlebar policy.
- tray popover remains startup/hidden/undecorated if needed.

### Manual Ubuntu Wayland QA

On Ubuntu Wayland installed package:

- Open settings.
- Hover minimize/maximize/close before touching window geometry.
- Click minimize.
- Reopen settings.
- Click maximize.
- Restore.
- Click close.
- Repeat for about.
- Repeat for update.
- Repeat for widget.
- Close and reopen each surface and repeat hover/click checks.
- Do not use the double-click top-center workaround during the pass.

Pass condition: every native control reacts on hover and click on first open and after close/reopen.

---

## Acceptance Criteria

The fix is accepted only when all are true:

- Ubuntu Wayland installed build: settings native controls work on first open.
- Ubuntu Wayland installed build: about native controls work on first open.
- Ubuntu Wayland installed build: update native controls work on first open.
- Ubuntu Wayland installed build: widget native controls work on first open.
- Close/reopen does not reintroduce the bug.
- Maximize/restore workaround is no longer needed.
- Widget background visually matches settings app-window background.
- Widget scrolling remains functional.
- `mochi diagnostics --bundle` shows the final creation path and contains no post-build decoration/resizable mutation calls for decorated Linux windows.
- `pnpm lint`
- `pnpm format:check`
- `pnpm test`
- `pnpm build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `cargo test --manifest-path src-tauri/Cargo.toml --all-targets`

---

## Non-Goals

- Do not redesign the settings UI.
- Do not change provider data fetching.
- Do not change updater feed behavior.
- Do not add custom Linux titlebar controls.
- Do not hide the native taskbar entry for settings/about/update/widget.

---

## Open Questions For Implementation

1. Does Tauri/Wry on Linux expose a reliable event corresponding to the first map/configure event that can be logged without adding GTK-specific code?
2. If visible-on-demand creation fixes the controls, should about/update remain routes inside the `settings` label or become separate decorated window labels to avoid pre-show size mutation?
3. Should widget be removed from `tauri.conf.json` entirely on Linux and created only by Rust, while macOS/Windows keep the config entry?
4. Should `RootComponent` treat widget as `isAppWindow || isWidgetWindow` for all document-level app-window shell attributes?
