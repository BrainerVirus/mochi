# Mochi app icon source

The desktop app icon uses **MochiChibi** (from `src/components/mascot/mochi-chibi.tsx`) on a **transparent canvas**.

## Design rationale

Do not paint a full-bleed background square. macOS, Windows, and Linux each apply native icon masks (squircle, tile, etc.) from bundled `.icns` / `.ico` assets at install time. A full opaque square bypasses that pipeline and reads as a harsh box in the Dock or taskbar.

| Element     | Choice                       | Why                                                  |
| ----------- | ---------------------------- | ---------------------------------------------------- |
| Background  | Transparent                  | OS applies mask and backdrop                         |
| Pet         | MochiChibi at ~82% safe zone | Centered with padding so details survive small sizes |
| Brand color | Pet cream/blush only         | Matcha Calm stays in-app, not on the tile            |

## Files

| Path                  | Purpose                                   |
| --------------------- | ----------------------------------------- |
| `mochi-app-icon.svg`  | Source artwork (1024×1024 viewBox)        |
| `../src-tauri/icons/` | Generated Tauri bundle (do not hand-edit) |

## Regenerate

```bash
./scripts/generate-icons.sh
```

## Verification

- **Installed builds:** OS-native icon treatment from `icon.icns` / `icon.ico`.
- **Dev (`pnpm tauri dev`):** macOS re-applies PNG from `src-tauri/icons/` when app windows open; packaged installs are the reference for final appearance.
