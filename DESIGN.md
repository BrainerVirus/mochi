# Design System: Mochi

**Project ID:** mochi-desktop-v1

## 1. Visual Theme & Atmosphere

Mochi is a **calm usage companion** for AI coding tools. It should feel like **part of the host OS** — a native menu-bar popover on macOS, a Fluent-adjacent flyout on Windows, and an Adwaita-friendly panel on Linux — not a custom dark skin pasted over every surface.

**One React/Tauri codebase.** Platform character comes from CSS scoped by `data-platform` on `<html>` (set via Tauri `get_platform`) plus `prefers-color-scheme` for light/dark. Do not fork UI per OS in TypeScript unless behavior truly differs.

**Key characteristics:**

- **Tray panel:** System background/text colors; light and dark follow OS preference. Mochi pastel usage meters (matcha → yuzu → ume) remain the brand accent on meters and the mark ring only.
- **Settings / About:** Full windows with native-adjacent surfaces; subtle Mochi warmth in light mode (cream-tinted shadcn tokens) while respecting system dark mode via `.dark` on `<html>`.
- **Density:** Compact but breathable — preserve approved tray layout tokens in `tray-panel-spacing.ts`; do not regress tab cycling, scroll fades, GSAP height morph, or progress bar animation.
- **Typography:** Platform system UI stacks in tray and app chrome; Geist remains available as fallback in the global theme.
- **Icons:** Prefer system-native metaphors (SF Symbol–style on macOS where feasible); monochrome provider glyphs in the menu bar.

## 2. Platform Guidance

### macOS

- **Font:** `-apple-system`, BlinkMacSystemFont, SF Pro Text
- **Surfaces:** Light `#f5f5f7`, dark `#323232` — native `NSVisualEffectMaterial::Popover` via `window-vibrancy` in Rust; webview layer stays transparent on macOS
- **Radius:** ~10px tray panel (`--radius-tray-panel: 0.625rem`)
- **Accent:** System blue `#007aff` / `#0a84ff` for focus rings and primary actions in tray
- **Color scheme:** `prefers-color-scheme` drives tray and app dark mode

### Windows

- **Font:** Segoe UI Variable, Segoe UI
- **Surfaces:** Fluent light `#f3f3f3`, dark `#202020`
- **Radius:** ~8px tray panel
- **Accent:** Fluent blue `#005fb8` / `#60cdff` (high contrast in dark)
- **Spacing:** Slightly roomier hit targets in footer menu rows

### Linux

- **Font:** system-ui, Cantarell, Ubuntu, Noto Sans (Adwaita-friendly)
- **Surfaces:** Light `#fafafa`, dark `#242424`
- **Radius:** ~6px tray panel — sharpest of the three
- **Accent:** Adwaita blue `#3584e4` / `#62a0ea`
- **Portals:** Keep styling portal-compatible (no macOS-only private APIs in shared CSS)

### Implementation hooks

| Hook                                           | Purpose                                                               |
| ---------------------------------------------- | --------------------------------------------------------------------- |
| `<html data-platform="macos\|windows\|linux">` | Platform typography, radius, accent overrides                         |
| `@media (prefers-color-scheme: dark)`          | Tray `.tray-panel` via `light-dark()`; app windows via `.dark` class  |
| `.tray-panel`                                  | Scoped semantic tokens — **not** forced always-dark                   |
| `.app-window`                                  | Settings/about shell — system surfaces + Mochi primary for brand CTAs |

Rust command: `get_platform` → frontend `detectPlatform()`.

## 3. Color Palette & Roles

### Mochi brand (usage meters & mark only)

| Name          | Hex     | Role                                |
| ------------- | ------- | ----------------------------------- |
| Matcha Calm   | #A3D9A5 | Healthy usage (<60%)                |
| Yuzu Glow     | #FFE4A1 | Warning (60–85%)                    |
| Ume Alert     | #FF8A8A | Critical (>85%), errors             |
| Lavender Rest | #C4B5E0 | Reset-soon, secondary accent        |
| Blush Mochi   | #FFB5C2 | Soft highlight (settings marketing) |
| Grilled Peach | #FFB088 | Settings primary CTA (light mode)   |

### Structural (semantic CSS variables)

Tray and app windows use **`--background`**, **`--foreground`**, **`--muted`**, **`--border`**, etc. Platform blocks in `src/styles/index.css` map these to OS-like values. Do not hardcode charcoal `#171412` on the tray.

### Functional mapping (usage states)

| State            | Meter fill | Brand mark ring         |
| ---------------- | ---------- | ----------------------- |
| Normal (<60%)    | Matcha     | Muted sage arc          |
| Warning (60–85%) | Yuzu       | Yuzu arc                |
| Critical (>85%)  | Ume        | Ume arc, heavier stroke |
| Reset soon       | Lavender   | Lavender arc            |

## 4. Typography Rules

**Tray panel:** `var(--font-platform)` — system UI per OS, `text-sm` / `text-xs` / tabular nums for percentages.

**Settings / About:** Geist Variable with platform fallback in `--font-sans`.

**Hierarchy (unchanged density):**

- Panel title: semibold, ~1.125rem
- Provider name: medium, 1rem
- Usage labels: regular, 0.75rem, muted
- Percent values: medium, tabular nums

Never below ~11px effective size.

## 5. Component Stylings

### Tray panel shell

- **Class:** `tray-panel` on shell (`TrayPanelShell`)
- **Shape:** `rounded-[var(--radius-tray-panel)]` — platform-specific
- **Backdrop:** macOS uses Rust `window-vibrancy` (`Popover` material); Windows tries Mica then Acrylic; Linux uses CSS `backdrop-filter` + semi-transparent tint
- **Height:** `h-auto` shell sized to content; native window height sync includes `TRAY_PANEL_SHELL_CHROME_PX` (`pt-3`) — no `h-full` stretch gap below footer
- **Surface:** System background; `ring-1 ring-border`; whisper shadow
- **No nested Cards** — flat sections, `Separator` only

### Usage meters

- Pill tracks; Mochi matcha/yuzu/ume fills by threshold (unchanged logic)
- `h-1` compact, `h-1.5` expanded
- Show `% left` + reset countdown

### Settings forms

- shadcn `FieldGroup` + `Field`
- Semantic tokens; primary stays Grilled Peach in light, adapts in dark
- `.app-window` wrapper for dedicated Tauri windows

### Buttons

- Tray footer: native-adjacent ghost/list rows (existing `TrayMenuRow`)
- Settings: shadcn variants on semantic colors

## 6. Layout Principles (tray — preserved)

- Max ~360px wide; single integrated surface
- Top tabs: Overview + per-provider; scroll fade masks; chevron cycle
- Overview: 2×2 grid, bar chart, provider list
- Footer: Refresh, Settings, About, Quit with shortcuts
- Menu bar icon: provider glyph + remaining `%` label (not usage-colored dot)
- Spacing: use `tray-panel-spacing.ts` tokens — **do not change** without UX review

## 7. shadcn Token Mapping

| Role                  | Token                        |
| --------------------- | ---------------------------- |
| System/app background | `--background`               |
| Body text             | `--foreground`               |
| Elevated surface      | `--card`                     |
| Brand CTA (settings)  | `--primary`                  |
| Borders               | `--border`                   |
| Mochi usage colors    | Tailwind `mochi-*` utilities |
| Tray corner radius    | `--radius-tray-panel`        |

Custom Tailwind: `mochi-cream`, `mochi-blush`, `mochi-matcha`, `mochi-yuzu`, `mochi-peach`, `mochi-ume`, `mochi-lavender`, `rounded-mochi` (settings hero cards only — not tray shell).

## 8. Motion & Brand Mark

**MochiMark:** geometric quota ring; state via color only (matcha → yuzu → ume → lavender).

- GSAP for panel height morph and mark transitions
- `prefers-reduced-motion`: instant swaps
- Do not regress tray focus fix or meter fill animation

## 9. Do / Don't

| Do                                | Don't                                             |
| --------------------------------- | ------------------------------------------------- |
| Follow OS light/dark in tray      | Force always-dark `.tray-panel`                   |
| Set `data-platform` from Tauri    | Ship macOS-only CSS without Win/Linux equivalents |
| Keep Mochi colors on meters       | Paint entire tray in CodexBar charcoal            |
| Use system fonts in tray          | Load display fonts in compact popover             |
| Test on all three desktop targets | Assume vibrancy/blur on every platform            |
