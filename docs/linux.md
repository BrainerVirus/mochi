# Mochi On Linux

Mochi supports Linux through AppImage, `.deb`, `.rpm`, Flatpak, tray mode, desktop widget mode, CLI mode, and status-bar output.

## Tray Support

Tray availability depends on the desktop environment. If the tray is unavailable, use widget mode or status-bar mode.

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
