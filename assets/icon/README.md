# Mochi app icon source

The desktop app icon uses **MochiChibi** (the rice-cake blob mascot from `src/components/mascot/mochi-chibi.tsx`) on a **neutral white canvas** — not a full-bleed brand color.

## Design rationale

macOS (and Windows/Linux) apply their own icon masks at display time. A square source with full-bleed Matcha green reads as a harsh green block in the Dock squircle. The icon instead follows standard app-icon conventions:

| Element | Choice | Why |
| ------- | ------ | --- |
| Background | White → `#FAFAF9` → `#F5F5F4` radial | Matches OS-adjacent neutrals in `DESIGN.md`; reads clean under the macOS squircle mask |
| Pet | MochiChibi at ~82% safe zone | Centered with padding so eyes and blush survive mask clipping at 16–32 px |
| Brand color | Pet cream/blush only | Matcha Calm (`#A3D9A5`) stays on usage meters and mark ring — not the dock tile |

Source artwork is a **square PNG/SVG canvas**. Do not pre-render a squircle or rounded-rect mask; `icon.icns` layers are square and the OS applies the mask.

## Files

| Path                  | Purpose                                   |
| --------------------- | ----------------------------------------- |
| `mochi-app-icon.svg`  | Source artwork (1024×1024 viewBox)        |
| `../src-tauri/icons/` | Generated Tauri bundle (do not hand-edit) |

## Regenerate

From the repo root:

```bash
chmod +x scripts/generate-icons.sh
./scripts/generate-icons.sh
```

This runs `pnpm tauri icon assets/icon/mochi-app-icon.svg --output src-tauri/icons`, producing `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.icns`, and `icon.ico`.

Requires pnpm and the Tauri CLI (`@tauri-apps/cli` devDependency).

## Verification notes

- **`icon.icns`** includes the standard macOS iconset layers (`icon_16x16` … `icon_512x512@2x`). Confirm with `iconutil -c iconset -o /tmp/check.iconset src-tauri/icons/icon.icns`.
- **Dev vs bundled:** `pnpm tauri dev` runs the bare binary (no `.app` wrapper), so the Dock may show a generic dev glyph until you install a bundled build. Packaged `.app` / `.dmg` installs use `Info.plist` + `icon.icns` and macOS applies the squircle mask automatically.
