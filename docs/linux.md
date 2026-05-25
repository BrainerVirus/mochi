# Mochi On Linux

Mochi supports Linux through AppImage, `.deb`, `.rpm`, Flatpak, tray mode, desktop widget mode, CLI mode, and status-bar output.

## Tray Support

Tray availability depends on the desktop environment. If the tray is unavailable, use widget mode or status-bar mode.

**GNOME (Ubuntu 24.04 default):** install a system tray extension (for example [AppIndicator and KStatusNotifierItem Support](https://extensions.gnome.org/extension/615/appindicator-support/)), then log out and back in. Without it, the Mochi tray icon may not appear.

**Settings / tray panel look wrong (mostly transparent):** older builds used transparent GTK windows without native blur. Current builds use opaque windows on Linux with CSS “glass” styling. Reinstall from a recent release if you still see a hollow window with only the title bar visible.

**Workaround while the tray is missing:** open **Settings** from the app menu if available, or run `mochi` from a terminal and use the desktop **widget** (tray menu → Show widget, when the tray works on another machine) or [status-bar mode](#status-bar) below.

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

## Flatpak Updates

Flatpak builds use Flatpak-managed updates. Mochi shows the same update prompt and triggers the Flatpak update path when the user clicks Update.
