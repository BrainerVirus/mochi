# Linux (Ubuntu) discover-by-running evidence — 2026-09-03

Task 8 of the release-provider-hardening workstream. Every result below was
produced by running the actual debug binary (`src-tauri/target/debug/mochi`)
on this machine. Nothing is invented; items that cannot be scripted are
marked **PENDING-OWNER** with repro steps.

## Environment

| Field         | Value                                                                                                                       |
| ------------- | --------------------------------------------------------------------------------------------------------------------------- |
| OS            | Ubuntu 24.04.4 LTS (noble), kernel 6.8.0-136-generic                                                                        |
| Desktop       | GNOME (ubuntu:GNOME), Wayland session (`XDG_SESSION_TYPE=wayland`, `WAYLAND_DISPLAY=wayland-0`, XWayland via `DISPLAY=:0`)  |
| App build     | 0.2.6, debug profile, frontend served by Vite 8 dev server on `http://localhost:1420`                                       |
| Tray stack    | GNOME Shell 46 + `ubuntu-appindicators@ubuntu.com` extension; `org.kde.StatusNotifierWatcher` registered on the session bus |
| Node for Vite | v24.14.0 via fnm (`pnpm dev` wrapper enforces >=24)                                                                         |

Build note: GTK/WebKit dev packages are not installed system-wide on this
machine; cargo ran against a deb-extracted sysroot at
`/tmp/opencode/mochi-sysroot/root` (`PKG_CONFIG_PATH` + `RUSTFLAGS`), rebuilt
this session per the task-5-report recipe. Runtime `.so` files are present
system-wide, so the built binary runs normally. CI is the authoritative
validation environment.

## Launch

PASS. `src-tauri/target/debug/mochi` (with Vite dev server on :1420) starts,
registers the tray, boots the main window frontend, and stays alive.

```
[mochi] tray registered (id=mochi-tray)
[window.snapshot] main url=http://localhost:1420/ visible=false
[frontend.boot] label=main href=http://localhost:1420/ path=/ target=/ tauri=true
```

Repro: `cd <repo> && pnpm dev` (or `node_modules/.bin/vite --port 1420`), then
`src-tauri/target/debug/mochi`. The main panel starts hidden-to-tray by
design (`visible=false`); the popover appears on tray interaction.

## Tray icon presence (appindicator)

PASS. The tray icon registers on the session bus and GNOME shows it in the
top bar. Verified via StatusNotifierWatcher:

```
gdbus call --session --dest org.kde.StatusNotifierWatcher \
  --object-path /StatusNotifierWatcher \
  --method org.freedesktop.DBus.Properties.Get \
  org.kde.StatusNotifierWatcher RegisteredStatusNotifierItems
→ '...:1.652@/org/ayatana/NotificationItem/tray_icon_tray_app_mochi_tray'
```

The SNI object reports `Status = Active`, `Title = mochi`, `XAyatanaLabel =
'70%'` (usage label updates as refreshes land), and an icon PNG served from
`$XDG_RUNTIME_DIR/tray-icon/` (44x44 RGBA re-rendered per state, counter
increments observed across refreshes).

## Tray menu (right-click / appindicator menu)

PASS. All five menu items are present and every one was triggered over DBus
(`com.canonical.dbusmenu.Event <id> clicked`) and observed to work:

| Item id | Label             | Observed result                                                                                      |
| ------- | ----------------- | ---------------------------------------------------------------------------------------------------- |
| 2       | Open widget       | widget window created + focused (`label=widget` lifecycle, `frontend.boot target=/widget`)           |
| 3       | Refresh usage     | all enabled providers refetched (usage_latest timestamps updated; commandcode included once enabled) |
| 4       | Settings          | settings window created + booted at `/settings`                                                      |
| 5       | Check for updates | update route opened in app window; app stays alive; updater feed URL fixed this task (see Fixes)     |
| 7       | Quit Mochi        | process exits cleanly (pgrep count 0)                                                                |

Menu layout verified via `com.canonical.dbusmenu.GetLayout`:
`Open widget, Refresh usage, Settings, Check for updates, —, Quit Mochi`.

## Tray left-click behavior

PENDING-OWNER (visual). On Linux, `tray-icon`/libappindicator delegates all
pointer interaction to the GNOME Shell indicator: `TrayIconEvent::Click` is
never emitted by the GTK backend (`tray-icon` 0.23.1 `platform_impl/gtk` has
no event emission; `menu_on_left_click` is documented "Linux: Unsupported"),
and the SNI object exposes no `Activate`/`ContextMenu` methods for scripting.
What is verifiable: the menu opens on click (GNOME behavior) and every menu
action works (above). What needs eyes: whether the anchored popover (the
`main` panel) can ever appear on GNOME, or whether the menu is the only
surface. The panel itself was verified working via `MOCHI_DEV_SHOW_MAIN=1`
(see Fixes: positioner crash) — it shows, renders, and hides on blur.

Repro for owner: click the Mochi icon in the GNOME top bar; observe whether
the usage popover or only the menu appears.

## Main panel (popover) open/close/hide

PASS (behavioral, via dev helper). `MOCHI_DEV_SHOW_MAIN=1` opens the panel
centered; on blur it hides (`hide_on_blur -> ok`). Before this task's fix the
open call **panicked and killed the app** on Wayland (see Fixes #2).

## Widget window

PASS. Tray menu "Open widget" creates/focuses the widget window: lifecycle
logs show `label=widget` created with `linux_builder_decorations ok=true`,
frontend boots at `target=/widget`, size 360x373 logical. Repeated opens
reuse/focus the existing window. On GNOME Wayland wmctrl cannot enumerate
windows (no X11 list for native windows); window existence is proven by the
Rust-side lifecycle diagnostics.

## Settings window

PASS. Tray menu "Settings" creates the settings window on demand
(`linux-on-demand-visible` policy, decorations on, 520x513 logical), boots
`target=/settings`, and re-invoking the menu focuses the existing window
without creating duplicates.

## Provider refresh (GUI path)

PASS. Tray menu "Refresh usage" refreshed every enabled provider; verified
through the shared sqlite store:

```
sqlite3 ~/.local/share/app.mochi.Mochi/usage.sqlite3 \
  "SELECT provider, updated_at FROM usage_latest ORDER BY updated_at DESC;"
codex|2026-09-03T17:55:56…  cursor|…56:12  opencode-go|…56:13  commandcode|…56:18
```

All four report `health: ok` (also confirmed via
`mochi usage --provider commandcode --refresh --json`, task 7 path).

## Updater check (stable feed)

FAIL before this task → FIXED. `check_for_update` built its endpoint from a
hard-coded base `https://mochi-app.github.io/mochi/updates`, but the live
GitHub Pages feed is served at `https://brainervirus.github.io/mochi/updates`
(the org `mochi-app` hosts zero public repos; both feed URLs 404 there). The
`mochi-app` → `brainervirus` host migration (commit `c99528f`) updated
`tauri.conf.json` and workflows but missed this Rust constant, so every
in-app update check 404'd. Fixed with TDD (see Fixes #1); the live feed
serves 200 with `version: 0.2.6` and platform `linux-x86_64`:

```
curl https://brainervirus.github.io/mochi/updates/linux/x86_64/0.2.6/stable.json → 200
```

The channel is stable-only: `update_endpoint_for_channel` rejects any channel
except `stable` (tested), and `tauri.conf.json` points at `stable.json`.

## CLI surfaces

| Command                                           | Status      | Notes                                                                              |
| ------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------- |
| `mochi --version`                                 | PASS        | prints `mochi 0.2.6`                                                               |
| `mochi diagnostics`                               | PASS        | full env/diagnostics output (session type, env workarounds, config hint, log tail) |
| `mochi usage [--provider X] [--refresh] [--json]` | PASS        | live refresh verified in task 7 and re-verified this session                       |
| `mochi status-bar`                                | PASS        | emits waybar JSON `{"text":"Mochi 42%","tooltip":"Codex",…}`                       |
| `mochi config`                                    | FAIL (stub) | prints `CLI subcommand not yet implemented: Config …`, exit 2                      |
| `mochi status`                                    | FAIL (stub) | same, exit 2                                                                       |
| `mochi cost`                                      | FAIL (stub) | same, exit 2                                                                       |
| `mochi update check`                              | FAIL (stub) | same, exit 2                                                                       |

The four stubs predate this task (design lists them as core CLI commands).
Left as-is: they are feature work, not platform defects. Recorded for the
backlog.

## Single-instance behavior

FAIL. Two `mochi` processes run simultaneously with no guard:

```
pgrep -x mochi | wc -l   → 2   (after launching a second instance)
```

Each registers its own tray icon (`RegisteredStatusNotifierItems` showed two
`tray_icon_tray_app_mochi_tray` entries at one point) and both fight over the
same sqlite store and settings file. Root cause: `tauri-plugin-single-instance`
is not used at all (`src-tauri/src/lib.rs` builder registers only
opener/positioner/updater plugins).

Not fixed in this task: adding the plugin changes app lifecycle semantics
(second-launch → focus existing window) and deserves its own review +
cross-platform verification (macOS activation policy interplay). Recorded as
a follow-up; owner may also want a DBus-only guard on Linux.

## Notifications

PENDING-OWNER. `show_notifications: true` exists in settings, but no
notification plugin/crate is wired in the Rust shell (no `tauri-plugin-notification`,
no `notify-rust` in Cargo.toml), and no notification code path exists outside
settings storage. Repro for owner: enable a provider limit threshold, cross
it, observe whether any desktop notification appears (none is expected).

## Window decorations / controls on GNOME

PASS (as far as observable). Settings windows are created with native
decorations (`decorations=linux_builder_decorations ok=true` on both creation
sources); the linux-on-demand-visible policy means decorated windows are
never pre-created hidden — the known Ubuntu Wayland titlebar hit-region
failure mode. Close/minimize/maximize button behavior on Wayland needs
visual confirmation:

PENDING-OWNER (visual): click the window close button on the settings
window — the app should keep running in the tray (close = hide per
`should_prevent_exit_request` lifecycle), not quit.

Repro: open Settings from the tray menu, click the titlebar X, verify the
tray icon remains and re-opening Settings works.

## Fixes applied this task

1. **Updater feed host** — `src-tauri/src/updater/mod.rs`:
   `UPDATE_ENDPOINT_BASE` `mochi-app.github.io` → `brainervirus.github.io`,
   matching `tauri.conf.json`, `docs/releasing.md`, and the live Pages feed.
   TDD: endpoint-const test updated first (RED), constant flipped (GREEN).
   Every in-app "Check for updates" previously 404'd.

2. **Wayland startup/panel positioning crash** —
   `src-tauri/src/tray/panel.rs`: the positioner plugin panics
   (`current_monitor()?.unwrap()`, ext.rs:155) when the window's monitor
   cannot be resolved, which GNOME Wayland reports before the window is
   mapped. Any panel positioning before that point killed the entire app
   (repro: `MOCHI_DEV_SHOW_MAIN=1 src-tauri/target/debug/mochi` → panic in
   `show_tray_panel_centered` during setup, exit). Fix: pure
   `should_skip_positioning` guard (unit-tested: monitor present → position,
   `Ok(None)`/`Err` → skip and log `position_skipped_no_monitor`) applied to
   both `position_tray_panel` and `show_tray_panel_centered`; the panel shows
   at its default position instead of crashing, and positions normally once
   a monitor resolves.

Both fixes are in platform/app modules (`updater`, `tray`); no `#[cfg]`
added under `src-tauri/src/core/` (CA-09). macOS/Windows paths unchanged:
`should_skip_positioning` is platform-agnostic (a monitor is always
resolvable on mac/win; if it ever is not, skipping a move is correct there
too), and the updater constant is channel/host-only.

## Known quirks (accepted, with reason)

- **Tooltip**: `tray-icon` GTK backend `set_tooltip` is a no-op
  ("Linux: Unsupported"); GNOME shows only the `XAyatanaLabel` percentage.
  Platform trait of libappindicator, not actionable in-app.
- **Tray icon rect**: `rect()` always `None` on Linux; the cached-rect
  anchored-popover path only activates if a `TrayIconEvent` ever fires, which
  does not happen through appindicator. Fallback positioning (BottomRight /
  position-skip) covers it.
- **Left-click opens menu, not popover**: appindicator delegates pointer
  handling to the shell; both buttons open the menu. See tray section above.
- **CLI stubs** (`config`, `status`, `cost`, `update`): declared but
  unimplemented; not platform defects.
