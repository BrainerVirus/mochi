# Design System: Mochi

**Project ID:** mochi-desktop-v1

## 1. Visual Theme & Atmosphere

Mochi is a **soft, calm usage companion** for AI coding tools. The interface should feel like a friendly kitchen counter note — warm, approachable, and never alarming until it truly needs to be. Data density is **compact but breathable**: tray panels and widgets show the essentials first, with room to expand into detail.

The aesthetic is **Japanese confectionery-inspired minimalism**: rounded, pillowy surfaces, pastel accents, and cream backgrounds that read as gentle rather than clinical. Usage warnings escalate through color and mascot expression, not through harsh reds or modal overload.

**Key characteristics:**

- Warm cream canvas with whisper-soft shadows
- Pill-shaped badges and generously rounded cards (`rounded-mochi`, 1.5rem)
- Pastel usage meters that shift from matcha → yuzu → ume as limits approach
- Mascot-driven emotional feedback alongside numeric data
- Light-first with a warm dark mode for late-night coding sessions
- **Tray popover:** Always uses scoped `.tray-panel` dark charcoal theme (CodexBar-inspired density) with Mochi pastel meter accents — independent of app light/dark preference

## 2. Color Palette & Roles

### Foundation

- **Steamed Rice Cream** (#FFF8F0) — Primary background (`bg-background`, `bg-mochi-cream`). The default canvas for tray panel, widget, and settings.
- **Warm Parchment White** (#FFFFFF at 80–95% opacity) — Card and popover surfaces (`bg-card`). Slightly elevated from the cream background.
- **Soft Cocoa Text** (#334155) — Primary foreground (`text-foreground`). Readable without harsh black contrast.

### Brand Accents

- **Blush Mochi** (#FFB5C2) — Primary accent and default usage meter fill (`bg-mochi-blush`). Friendly highlight for active provider and normal usage states.
- **Grilled Peach** (#FFB088) — Primary action color (`bg-primary`). Warm CTA buttons and focused controls.
- **Matcha Calm** (#A3D9A5) — Success and healthy usage (`bg-mochi-matcha`). Under 60% usage, connected status, positive confirmations.
- **Yuzu Glow** (#FFE4A1) — Warning band (`bg-mochi-yuzu`). 60–85% usage, approaching limits, stale-but-usable data.
- **Ume Alert** (#FF8A8A) — Critical and destructive (`bg-mochi-ume`, `destructive`). 85%+ usage, errors, disconnects.
- **Lavender Rest** (#C4B5E0) — Secondary accent (`bg-mochi-lavender`). Reset-soon states, secondary badges, subtle highlights.

### Structural

- **Warm Stone Border** (#E7E0D8) — Borders and dividers (`border-border`). Barely-there separation on cream backgrounds.
- **Muted Sage Gray** (#64748B) — Secondary text (`text-muted-foreground`). Labels, timestamps, provider metadata.

### Functional Mapping (Usage States)

| State            | Color         | Mascot                   |
| ---------------- | ------------- | ------------------------ |
| Normal (<60%)    | Matcha Calm   | Happy mochi              |
| Warning (60–85%) | Yuzu Glow     | Worried mochi            |
| Critical (>85%)  | Ume Alert     | Flattened/sweating mochi |
| Reset soon       | Lavender Rest | Excited mochi + clock    |
| All good         | Blush Mochi   | Bouncy mochi             |

## 3. Typography Rules

**Primary font:** Geist Variable (sans-serif) — clean, modern, legible at small tray-panel sizes.

**Hierarchy:**

- **Panel title:** Semi-bold (600), 1.125rem, tight tracking
- **Provider name:** Medium (500), 1rem
- **Usage labels:** Regular (400), 0.75rem, muted foreground
- **Percent values:** Medium (500), 0.75rem, tabular nums
- **Tagline / brand:** Medium (500), 0.75rem, expanded letter-spacing (0.2em), uppercase

**Principles:**

- Prefer `text-sm` and `text-xs` in compact tray/widget surfaces
- Use tabular figures for percentages and countdowns
- Never below 11px effective size for accessibility

## 4. Component Stylings

### Buttons

- **Shape:** Generously rounded (`rounded-lg` to `rounded-full` for pill CTAs)
- **Primary:** Grilled Peach background, cream-white text
- **Secondary:** Warm Parchment with Warm Stone border
- **Ghost:** Transparent with muted hover on cream
- **Destructive:** Ume Alert at 10–20% opacity background, full ume text

### Cards & Containers

- **Shape:** `rounded-mochi` (1.5rem) for hero cards; `rounded-lg` for nested items
- **Background:** Warm Parchment White with 80% opacity option for overlay panels
- **Shadow:** Whisper-soft (`shadow-sm`), ring-1 in Warm Stone Border
- **Padding:** `p-4` compact, `p-6` for settings sections
- **Tray panel exception:** Do **not** use `Card` on the tray route — use flat sections on the integrated dark `.tray-panel` surface instead.

### Usage Meters

- **Track:** Muted warm gray (`bg-muted`) on light surfaces; `bg-muted` on tray dark surface
- **Fill:** State-driven — matcha → yuzu → ume gradient logic
- **Height:** `h-1` in tray/compact mode, `h-1.5` in expanded
- **Shape:** Pill track and fill (`rounded-full`)
- **Labels:** Show `% left` (not `% used`) plus reset countdown when `resets_at` is present

### Badges

- **Shape:** Pill (`rounded-full`)
- **Variants:** Secondary for source tags, outline for provider IDs, destructive tint for error/stale

### Inputs & Forms (Settings)

- **Stroke:** 1px Warm Stone Border
- **Background:** Steamed Rice Cream, shifts to white on focus
- **Focus ring:** Grilled Peach at 50% opacity
- **Layout:** `FieldGroup` + `Field` composition (shadcn forms)

### Alerts & Toasts

- **Update prompt:** Card-style alert with primary + ghost actions
- **Stale data:** Yuzu-tinted alert, never blocking
- **Errors:** Ume-tinted alert with retry action

## 5. Layout Principles

### Surfaces

- **Tray panel:** Max ~360px wide, **single integrated dark surface** (`.tray-panel` scoped theme), CodexBar-style information density. No nested cards — sections separated by `Separator` only.
- **Widget:** Resizable 280–480px, density modes adjust padding and font scale
- **Settings:** Max 720px centered, tabbed sections

### Tray Panel Layout (required patterns)

- **Single surface:** One charcoal panel shell (`tray-panel` class on `TrayPanelShell`); never stack `Card` inside the tray route.
- **Top tabs:** Horizontal `Tabs` with `variant="line"` — Overview plus one tab per enabled provider. Each provider tab shows a mini usage bar (`h-0.5` Progress) tinted by usage state.
- **Overview tab:** 2×2 metric grid (providers, highest %, average %, healthy count), compact bar chart, then flat provider meter list.
- **Provider tab:** Flat section with thin meters, `% left` + reset countdown, source badge — no card wrapper.
- **Header:** Minimal — wordmark, refresh icon, settings gear. No mascot in tray panel (data-first).
- **Typography:** Geist at `text-sm` / `text-xs` / `text-[10px]` labels; tabular nums for percentages.
- **Meters:** Thin tracks (`h-1`), Mochi matcha/yuzu/ume fill by threshold; label row shows window name, `% left`, and reset time when available.
- **Spacing:** `px-3` horizontal padding, `gap-3` between sections, `Separator` between logical groups.

### Spacing

- Base unit: 4px
- Component gap: `gap-3` (12px) compact, `gap-4` (16px) normal
- Section margin: `gap-6` between provider groups
- Use `gap-*`, never `space-y-*`

### Density Modes

- **Compact:** `text-xs`, `p-3`, single-line meters
- **Normal:** `text-sm`, `p-4`, standard meters + mascot thumbnail
- **Expanded:** `text-base`, `p-6`, dual meters + provider detail link

## 6. shadcn Token Mapping

When building UI, map Mochi semantics to shadcn tokens:

| Mochi role           | shadcn token                 |
| -------------------- | ---------------------------- |
| Steamed Rice Cream   | `--background`               |
| Soft Cocoa Text      | `--foreground`               |
| Warm Parchment White | `--card`                     |
| Grilled Peach        | `--primary`                  |
| Blush Mochi          | `--accent`                   |
| Warm Stone Border    | `--border`                   |
| Muted Sage Gray      | `--muted-foreground`         |
| Ume Alert            | `--destructive`              |
| 1.5rem corners       | `--radius` / `rounded-mochi` |

Custom Tailwind tokens (always available): `mochi-cream`, `mochi-blush`, `mochi-matcha`, `mochi-yuzu`, `mochi-peach`, `mochi-ume`, `mochi-lavender`, `rounded-mochi`.

## 7. Motion & Mascot

- **GSAP** for mascot state transitions and panel enter/exit
- **Reduced motion:** Instant state swaps, no bounce
- **Micro-interactions:** 200–300ms ease on meter fill width changes
- Mascot sits beside or above usage summary; never obscures data
