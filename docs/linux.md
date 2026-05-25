# Mochi On Linux

Mochi supports Linux through **AppImage**, **`.deb`**, and **`.rpm`** packages, plus tray mode, desktop widget mode, CLI mode, and status-bar output. **Flatpak is not supported** — native packages avoid sandbox limits on browser cookie import and keep updates on the Tauri updater path.

## Install

The one-line installer sets up runtime dependencies (tray indicator, libsecret, SVG) before downloading Mochi:

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-linux.sh | bash
```

The installer resolves `main` to the latest commit SHA so GitHub’s raw CDN does not serve a stale `common.sh`. You should see `Mochi_0.1.3_amd64.deb` in the download line, not `Mochi_x64.app.tar.gz`. If an old script is cached, pin the install ref: `MOCHI_INSTALL_REF=097112d curl -fsSL ... | bash`.

**Stable (recommended):** installs the latest non-prerelease tag (use **v0.1.3+** for the Linux blank-window fix; avoid **v0.1.2** on Linux).

```bash
MOCHI_VERSION=v0.1.3 curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-linux.sh | bash
```

**Unstable** (`-i` uses the `unstable` release tag, not deprecated v0.1.0):

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-linux.sh | bash -s -- -i
```

Set `MOCHI_SKIP_DEPS=1` to skip the dependency phase. See [README](../README.md) for package formats and pinning releases.

## Tray Support

Tray availability depends on the desktop environment. If the tray is unavailable, use widget mode or status-bar mode.

**GNOME (Ubuntu 24.04 default):** the install script tries to install and enable an AppIndicator extension (for example [AppIndicator and KStatusNotifierItem Support](https://extensions.gnome.org/extension/615/appindicator-support/)). **Log out and back in** after install if the tray icon still does not appear.

**Settings / tray panel look wrong (mostly transparent):** older builds used transparent GTK windows without native blur. Current builds use opaque windows on Linux with CSS “glass” styling. Reinstall from a recent release if you still see a hollow window with only the title bar visible.

**Workaround while the tray is missing:** open **Settings** from the app menu if available, use the desktop **widget** from the tray menu when the tray works, or run `mochi` from a terminal and use [status-bar mode](#status-bar) below. Settings shows a Linux note when the tray may be unavailable.

## Browser cookie import

Auto-import reads Chromium and Firefox/Zen profiles under `~/.config` (and GNOME Keyring / KWallet via libsecret). Unlock your keyring when prompted. Safari import remains macOS-only.

## Status Bar

Waybar example:

```json
{
  "custom/mochi": {
    "exec": "mochi status-bar --format waybar",
    "interval": 60,
    "return-type": "json"
  }
}
```

## Diagnostics

If settings or the widget open as a blank window, or the close button does nothing, collect a redacted report and attach it to a GitHub issue.

**After reproducing the problem** (open settings and/or widget, try close/minimize), run:

```bash
# Installed .deb / AppImage (binary on PATH)
mochi diagnostics
mochi diagnostics --bundle

# If `mochi` is not on PATH, use the full path, e.g.:
# /usr/bin/mochi diagnostics --bundle
# ~/.local/bin/mochi diagnostics --bundle
```

Copy the terminal output (or the bundle path printed by `--bundle`) into your issue. The bundle is a text file under `~/.local/share/app.mochi.Mochi/logs/` (filename like `mochi-diagnostics-*.txt`).

Optional: include the log tail directly:

```bash
tail -n 120 ~/.local/share/app.mochi.Mochi/logs/diagnostics.log
```

Launching from a terminal also prints diagnostic lines to stderr:

```bash
mochi
# or: /usr/bin/mochi
```

The desktop app writes `diagnostics.log` while it runs. Reports include version, platform, webview URLs, frontend boot routes, and recent events (paths and secrets are redacted).

## Updates

Linux **AppImage**, **`.deb`**, and **`.rpm`** builds use the in-app Tauri updater (same flow as macOS and Windows). Re-run the install script or use your package manager to upgrade pinned releases.

If you previously used Flatpak, uninstall it and switch to `install-linux.sh` (deb on Ubuntu/Debian, AppImage elsewhere).
