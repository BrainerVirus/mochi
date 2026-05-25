# Mochi app icon source

The desktop app icon uses **MochiChibi** (the rice-cake blob mascot from `src/components/mascot/mochi-chibi.tsx`) on a **full-bleed** Matcha Calm background (`#A3D9A5` from `DESIGN.md`).

## Files

| Path | Purpose |
| --- | --- |
| `mochi-app-icon.svg` | Source artwork (1024×1024 viewBox) |
| `../src-tauri/icons/` | Generated Tauri bundle (do not hand-edit) |

## Regenerate

From the repo root:

```bash
chmod +x scripts/generate-icons.sh
./scripts/generate-icons.sh
```

This runs `pnpm tauri icon assets/icon/mochi-app-icon.svg --output src-tauri/icons`, producing `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.icns`, and `icon.ico`.

Requires pnpm and the Tauri CLI (`@tauri-apps/cli` devDependency).

## Design notes

- Background fills the entire square — no transparent margins.
- Pet is scaled to ~80% of the canvas so it reads clearly at menu-bar and dock sizes.
- Colors: Matcha Calm gradient background; pet uses the same cream/blush palette as the React SVG.
