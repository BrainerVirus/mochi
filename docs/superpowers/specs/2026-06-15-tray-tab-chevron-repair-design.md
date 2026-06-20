# Tray Tab Chevron Crash & Active-Pill Animation Repair

**Date:** 2026-06-15
**Status:** Design approved, pending implementation

## Overview

Two related defects affect the Mochi tray page-tab strip (used by the tray panel and the widget window):

| #   | Issue                                                                                    | Established behavior                                                                                                                                                                                                                                                                                                                                                                                                             |
| --- | ---------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Rapid clicks on the tray tab chevron controls crash the app (window closes, no recovery) | Rapid `visible` toggles repeatedly reconstruct the chevron's `useGSAP` and `gsap.matchMedia()` visibility setup. A renderer crash was observed during rapid interaction, while an unmemoized `handleTabChange` and duplicated `syncTrayUsage` IPC call were also present. The available evidence does not establish the crash's exact mechanism. CSS transitions remove the unnecessary GSAP reconstruction and allocation path. |
| 2   | The active tab pill does not animate when the user cycles tabs via the chevron           | The chevron path in `cycleTrayPanelTabs` calls the parent's `onValueChange` directly, bypassing the segmented control's `handleValueChange` which is the only path that dispatches the machine's `SELECT` → `moveActive` command. Tray page tabs are machine-driven (the layout-effect path is intentionally disabled for tray to preserve hover handoff).                                                                       |

The repair lifts the segmented control's state machine to the tab list parent (so both click paths go through the same handler), replaces the chevron's GSAP animation with CSS transitions, and removes the IPC duplication. These changes remove repeated visibility reconstruction and redundant work from the interaction path without asserting an unproven crash mechanism.

---

## Issue 1: Lift the State Machine to the Parent (Fixes the Animation)

### Problem

Direct tab click path:

```
ToggleGroup.onValueChange
  → AppSegmentedControl → state.handleValueChange
    → useAppSegmentControlState.handleValueChange
      → useTraySegmentIndicators.handleSegmentValueChange
        → handleSelect(next)        ← machine SELECT → moveActive
        → onValueChange(next)       ← parent callback (store update)
```

Chevron click path:

```
TrayTabChevron.onClick
  → cycleTrayPanelTabs(scrollEl, tabs, value, "forward", onValueChange, ...)
    → onValueChange(nextTab.id)    ← parent callback only, NO machine
```

For tray (`showHover: true`), `syncOnValueChange` is `false` (`use-tray-segment-indicators.ts:136`), so the layout effect at `use-tray-segment-indicator-sync.ts:155-157` returns early. The active pill animation is machine-driven only, and the chevron path never dispatches `moveActive`. The pill is left at its old position visually.

### Solution: M3 — Lift `useAppSegmentControlState` to the Parent

The `useAppSegmentControlState` hook is the owner of the state machine. It creates refs (`trackRef`, `activeIndicatorRef`, `hoverIndicatorRef`, `itemRefs`) and the `handleValueChange` that wraps the machine. Lifting it to `TrayPanelTabList` gives the parent access to the same `handleValueChange` that the segmented control uses internally, so the chevron dispatches the same machine path as a direct tab click.

The challenge is ref ownership: refs must be attached to DOM elements rendered by the component that creates them. The fix is to split rendering from control.

#### 1a. Extract `AppSegmentedControlView`

New file: `src/components/ui/app-segmented-control-view.tsx`

A pure presentational component. Accepts:

```typescript
interface AppSegmentedControlViewProps {
  state: ReturnType<typeof useAppSegmentControlState>;
  items: AppSegmentItem[];
  value: string;
  onValueChange: (value: string) => void;
  className?: string;
  rowHeight?: string;
  stretchItems?: boolean;
  variant?: AppSegmentedControlVariant;
  layout?: AppSegmentedControlLayout;
  contentReady?: boolean;
}
```

Behavior:

- Attaches `state.trackRef` to the track `<div>`.
- Attaches `state.activeIndicatorRef` and `state.hoverIndicatorRef` to the indicator `<div>`s.
- Wires `state.setItemRef` and `state.syncHoverIndicator` into the toggle items.
- Uses `state.handleValueChange` for `ToggleGroup`'s `onValueChange`.
- Uses `state.pillReady` for the `blockInteractionUntilPlaced` gate.
- All other rendering logic is moved verbatim from `app-segmented-control.tsx`.
- Imports `useAppSegmentControlState` solely for the `ReturnType<typeof ...>` type expression. No runtime call. (Alternative: export the return type as a named alias from `use-app-segment-control-state.ts` and import the alias instead. Pick one and apply consistently.)

#### 1b. Slim Down `AppSegmentedControl`

File: `src/components/ui/app-segmented-control.tsx`

Becomes a thin backward-compatible wrapper:

```typescript
export function AppSegmentedControl(props: AppSegmentedControlProps) {
  const state = useAppSegmentControlState(
    props.value,
    props.items.length,
    props.onValueChange,
    {
      enabled: usesSegmentActiveIndicator(props.variant ?? "page-tabs"),
      showHover: usesSegmentHoverIndicator(
        props.variant ?? "page-tabs",
        props.layout ?? "tray",
      ),
      contentReady: props.contentReady ?? true,
    },
  );
  return <AppSegmentedControlView state={state} {...props} />;
}
```

Inline and settings callers (which currently use `AppSegmentedControl` directly) continue to work unchanged. `SettingsTabSegmentedControl` in `tray-segmented-control.tsx` continues to wrap `AppSegmentedControl`.

#### 1c. Lift the Hook in `TrayPanelTabList`

File: `src/features/tray/components/tray-panel-tab-list/tray-panel-tab-list.tsx`

```typescript
export function TrayPanelTabList({ tabs, value, onValueChange }: TrayPanelTabListProps) {
  const items = useMemo(
    () => tabs.map((tab) => ({
      id: tab.id,
      label: tab.label,
      icon: tab.id === "overview"
        ? <LayoutGridIcon aria-hidden />
        : <ProviderIcon provider={tab.id} />,
    })),
    [tabs],
  );

  const tabControlState = useAppSegmentControlState(
    value,
    items.length,
    onValueChange,
    { enabled: true, showHover: true, contentReady: true },
  );

  const handleCycleForward = useCallback(
    (scrollEl: HTMLDivElement) => {
      cycleTrayPanelTabs(
        scrollEl,
        tabs,
        value,
        "forward",
        tabControlState.handleValueChange,
        TAB_FADE_INSET,
      );
    },
    [tabControlState.handleValueChange, tabs, value],
  );

  const handleCycleBackward = useCallback(
    (scrollEl: HTMLDivElement) => {
      cycleTrayPanelTabs(
        scrollEl,
        tabs,
        value,
        "backward",
        tabControlState.handleValueChange,
        TAB_FADE_INSET,
      );
    },
    [tabControlState.handleValueChange, tabs, value],
  );

  return (
    <div
      data-tray-tab-strip
      className="border-border min-w-0 overflow-hidden rounded-t-[var(--radius-tray-panel)] border-b px-3 pb-2"
    >
      <ScrollFadeRegion
        orientation="horizontal"
        className="w-full min-w-0 overflow-hidden"
        rowHeightClassName={TRAY_SEGMENT_ROW_HEIGHT}
        scrollClassName="overscroll-x-contain"
        fadeInset={TAB_FADE_INSET}
        onCycleForward={handleCycleForward}
        onCycleBackward={handleCycleBackward}
      >
        <AppSegmentedControlView
          state={tabControlState}
          items={items}
          value={value}
          onValueChange={onValueChange}
          variant="page-tabs"
          layout="tray"
          rowHeight={TRAY_SEGMENT_ROW_HEIGHT}
          stretchItems={false}
        />
      </ScrollFadeRegion>
    </div>
  );
}
```

`tabs` and `value` continue to drive the `useCallback` chain. The `useCallback` deps for `handleCycleForward` include `tabControlState.handleValueChange`. That handler's stability is what determines the chevron callback's stability: it is stable iff `value` and `onValueChange` are stable. `onValueChange` is the parent's `handleTabChange`, which is memoized in Issue 3. `value` comes from the Zustand store and only changes when the store changes. Net result: `handleCycleForward` is referentially stable across renders that don't change `tabs` or `value`.

#### 1d. Move `SettingsTabSegmentedControl` to the Settings Feature

After 1c, `TraySegmentedControl` and `PageTabSegmentedControl` have no callers (`PageTabSegmentedControl` is only used internally by `TraySegmentedControl`; `TraySegmentedControl` is only used by `tray-panel-tab-list.tsx`). Both are deleted.

`SettingsTabSegmentedControl` stays, but moves out of the tray feature so the file path is not misleading:

- **New file**: `src/features/settings/components/settings-tab-segmented-control/settings-tab-segmented-control.tsx`. Contains `SettingsTabSegmentedControl`, which wraps `AppSegmentedControl` with `SETTINGS_PAGE_TAB_DEFAULTS`.
- **New file**: `src/features/settings/components/settings-tab-segmented-control/settings-tab-segmented-control-config.ts`. Contains `SETTINGS_PAGE_TAB_DEFAULTS` and `SETTINGS_SEGMENT_INDICATOR_RADIUS_CLASS` / `SETTINGS_SEGMENT_TRACK_RADIUS_CLASS` if they were co-located. (Check `tray-segmented-control-config.ts` for the exact constants and move the settings-specific ones.)
- **Delete file**: `src/features/tray/components/tray-segmented-control/tray-segmented-control.tsx`. All exports moved or removed.
- **Update import**: `src/features/settings/components/settings-form/settings-form.tsx:3` — change the import path to the new settings-folder location.
- **Keep in tray feature**: `TRAY_SEGMENT_ROW_HEIGHT` and `TRAY_PAGE_TAB_DEFAULTS` stay in `src/features/tray/components/tray-segmented-control-config/` (used by `tray-panel-tab-list.tsx`).

The new per-unit folder layout follows the convention from `docs/tech-stack.md`: one folder per unit, no `index.ts` barrels, test colocated as `settings-tab-segmented-control.test.tsx`.

---

## Issue 2: Replace Chevron GSAP with CSS Transitions

### Problem

`tray-tab-chevron.tsx:33-61` and `scroll-fade-overlays.tsx:31-41` use:

```typescript
useGSAP(
  () => {
    const column = columnRef.current;
    if (!column) return undefined;
    const mm = gsap.matchMedia();
    mm.add("(prefers-reduced-motion: reduce)", () => {
      /* ... */
    });
    mm.add("(prefers-reduced-motion: no-preference)", () => {
      /* ... */
    });
    return () => {
      mm.revert();
    };
  },
  { dependencies: [hiddenX, visible], scope: columnRef, revertOnUpdate: true },
);
```

Every `visible` toggle reconstructs the `gsap.matchMedia()` visibility setup and reverts the previous setup during cleanup. A renderer crash was observed during rapid toggles, but the available evidence does not establish asynchronous cleanup, stacked contexts, a memory leak, or an out-of-memory failure as the cause.

The vertical chevron in `scroll-fade-overlays.tsx` shares the same pattern via `use-gsap-overflow-visibility.ts` and is equally affected.

### Solution: G1 — CSS Transitions

The chevron animation is a 0.2s opacity + 4px translateX (or translateY for vertical) fade. CSS `transition` does the same thing natively:

- `transition: opacity 0.2s ease-out, transform 0.2s ease-out` matches the GSAP `power2.out` easing closely enough for a chevron.
- The browser handles interruption (a new value mid-transition starts a new tween from the current value) — equivalent to GSAP's `overwrite: "auto"`.
- `motion-reduce:transition-none` (Tailwind) handles the reduced-motion media query at the CSS layer.
- No `mm` context reconstruction or allocation per visibility update.

Interaction gating: the column always has `pointer-events-none` (so clicks pass through to the underlying button). The button uses `getTrayTabChevronButtonClassName(visible)` which already returns `pointer-events-none` when `!visible`, plus `tabIndex={visible ? 0 : -1}`. The `invisible` Tailwind class (`visibility: hidden`) is dropped from the column — `opacity-0` plus the button's `pointer-events-none` is sufficient to keep the chevron non-interactive while hidden. `aria-hidden={!visible}` remains for assistive tech.

#### 2a. `src/features/tray/components/tray-tab-chevron/tray-tab-chevron.tsx`

Remove imports:

```typescript
import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import {
  SCROLL_OVERFLOW_FADE_DURATION_S,
  SCROLL_OVERFLOW_FADE_EASE,
  SCROLL_OVERFLOW_SLIDE_PX,
} from "@/features/tray/components/use-gsap-overflow-visibility";
```

Remove the `gsap.registerPlugin(useGSAP)` call. Remove the `useGSAP(...)` block and the `columnRef` (no longer needed; the column's animation is CSS-driven). Remove the `hiddenX` calculation and the `TrayTabChevronSide` import (the translate direction is now a className condition).

Replace the column `<div>` with:

```tsx
<div
  className={cn(
    "pointer-events-none absolute inset-y-0 z-30 flex w-8 items-center justify-center",
    "transition-[opacity,transform] duration-200 ease-out motion-reduce:transition-none",
    isStart ? "left-0" : "right-0",
    !visible ? "opacity-0" : "opacity-100",
    !visible && (isStart ? "-translate-x-1" : "translate-x-1"),
  )}
  aria-hidden={!visible}
>
```

The button inside is unchanged.

#### 2b. `src/features/tray/components/scroll-fade-overlays/scroll-fade-overlays.tsx`

Same treatment for `ScrollFadeVerticalChevron`:

- Drop `useGSAP` and `animateOverflowVisibility` imports.
- Drop `gsap.registerPlugin(useGSAP)`.
- Drop `SCROLL_OVERFLOW_SLIDE_PX` import.
- Replace the column classes with `transition-[opacity,transform] duration-200 ease-out motion-reduce:transition-none`.
- Toggle `opacity-0` and `translate-y-1` (or `-translate-y-1` for start side) based on `visible`.
- The button is unchanged.

#### 2c. `src/features/tray/components/use-gsap-overflow-visibility/`

After 2a and 2b, this folder has no remaining importers (verified by grep — the only consumers are the two chevron files). Delete the folder. Check the folder for a colocated test file (`.test.ts`); if present, delete it too. The folder contains only `use-gsap-overflow-visibility.ts` (no test file in the current tree, but verify before deletion). If a future caller needs it, restore from git history.

---

## Issue 3: Stability Fixes (Reduces Rapid-Click Work)

These are independent stability improvements that reduce work performed by the rapid-click path and should ship in the same change.

### 3a. Memoize `handleTabChange`

File: `src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.ts`

```typescript
const handleTabChange = useCallback(
  (value: string) => {
    const nextTab = parseTrayTabChange(value);
    setSelectedTab(nextTab);

    if (settings) {
      void persistTabChangeSettings(queryClient, settings, nextTab);
    }
  },
  [settings, setSelectedTab, queryClient],
);
```

`setSelectedTab` and `queryClient` are stable refs (Zustand selector and `useQueryClient()`), so `handleTabChange` is referentially stable per-settings-snapshot. This eliminates the stale-closure window in `TrayPanelTabList`'s `useCallback` chain: every chevron click now sees the current `tabs` and `value`.

### 3b. Remove the Duplicated `syncTrayUsage`

Same file. The synchronous `void syncTrayUsage(nextTab)` inside `handleTabChange` is redundant: the existing effect at lines 62-64 already calls `syncTrayUsage(selectedTab)` whenever `selectedTab` changes. Removing the duplicate halves IPC traffic per click and prevents the IPC queue from growing under rapid clicks.

---

## Files Changed

| File                                                                                                       | Change                                                                                                                                                                    |
| ---------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/components/ui/app-segmented-control-view.tsx`                                                         | **New.** Pure presentational, takes `state` as a prop.                                                                                                                    |
| `src/components/ui/app-segmented-control.tsx`                                                              | Slim down: call `useAppSegmentControlState`, render `AppSegmentedControlView`. Backward compatible.                                                                       |
| `src/components/ui/app-segmented-control-view.test.tsx`                                                    | **New.** Assert refs/handlers wire to the right DOM nodes with a mock state.                                                                                              |
| `src/components/ui/use-app-segment-control-state.test.ts`                                                  | **New.** Assert `handleValueChange` wraps the machine when `enabled` and skips it otherwise.                                                                              |
| `src/components/ui/app-segmented-control.test.ts`                                                          | **Unaffected.** Existing tests cover utility functions in `app-segmented-control-utils.ts`, not the component. No change needed.                                          |
| `src/features/tray/components/tray-panel-tab-list/tray-panel-tab-list.tsx`                                 | Lift `useAppSegmentControlState`; render `AppSegmentedControlView`; pass `tabControlState.handleValueChange` to `cycleTrayPanelTabs`.                                     |
| `src/features/tray/components/tray-panel-tab-list/tray-panel-tab-list.test.tsx`                            | **New.** Assert the chevron's `onCycle` invokes `state.handleValueChange`; rapid clicks don't throw.                                                                      |
| `src/features/tray/components/tray-panel-tab-cycle/tray-panel-tab-cycle.test.ts`                           | **New.** Branches: `currentIndex === -1` early return; `nextIndex === currentIndex` scroll-only; rapid sequential calls; stale `currentValue`.                            |
| `src/features/tray/components/tray-segmented-control/tray-segmented-control.tsx`                           | **Delete.** All exports (`TraySegmentedControl`, `PageTabSegmentedControl`) are removed; `SettingsTabSegmentedControl` moves to the settings feature (see below).         |
| `src/features/settings/components/settings-tab-segmented-control/settings-tab-segmented-control.tsx`       | **New.** Contains `SettingsTabSegmentedControl`, wrapping `AppSegmentedControl` with `SETTINGS_PAGE_TAB_DEFAULTS`.                                                        |
| `src/features/settings/components/settings-tab-segmented-control/settings-tab-segmented-control-config.ts` | **New.** Contains `SETTINGS_PAGE_TAB_DEFAULTS` (and any settings-specific radius/track constants that were co-located in `tray-segmented-control-config.ts`).             |
| `src/features/settings/components/settings-tab-segmented-control/settings-tab-segmented-control.test.tsx`  | **New.** Smoke test that the control renders with the settings layout.                                                                                                    |
| `src/features/settings/components/settings-form/settings-form.tsx`                                         | Update the import path of `SettingsTabSegmentedControl` from the tray feature to the new settings feature location.                                                       |
| `src/features/tray/components/tray-segmented-control-config/`                                              | **Trim.** Remove `SETTINGS_PAGE_TAB_DEFAULTS` (moved to settings). Keep `TRAY_SEGMENT_ROW_HEIGHT` and `TRAY_PAGE_TAB_DEFAULTS` (still used by `tray-panel-tab-list.tsx`). |
| `src/features/tray/components/tray-tab-chevron/tray-tab-chevron.tsx`                                       | Replace `useGSAP` + `gsap.matchMedia()` with CSS transitions. Drop `use-gsap-overflow-visibility` imports. Remove `columnRef` and the `useGSAP` block.                    |
| `src/features/tray/components/tray-tab-chevron/tray-tab-chevron.test.ts`                                   | **Extend.** Assert CSS transition utility classes are present; no `useGSAP` / `gsap` / `@gsap/react` imports remain; reduced-motion class is present.                     |
| `src/features/tray/components/scroll-fade-overlays/scroll-fade-overlays.tsx`                               | Same treatment for the vertical chevron. Drop `use-gsap-overflow-visibility` imports.                                                                                     |
| `src/features/tray/components/use-gsap-overflow-visibility/`                                               | **Delete.** No remaining importers after 2a/2b.                                                                                                                           |
| `src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.ts`                                     | Memoize `handleTabChange` with `useCallback`. Remove synchronous `syncTrayUsage` from `handleTabChange` (the effect at lines 62-64 already covers it).                    |
| `src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.test.ts`                                | **Extend.** Assert `handleTabChange` is referentially stable across renders with the same settings, and that it does not call `syncTrayUsage` directly.                   |
| `src/features/tray/components/tray-segment-indicator/tray-segment-indicator.test.ts`                       | **Extend.** Assert `moveActive` is dispatched on every value change regardless of which path triggered it.                                                                |
| `src/features/tray/components/tray-segment-indicator-machine/tray-segment-indicator-machine.test.ts`       | **Extend.** Same — assert machine `SELECT` emits `moveActive` (regression guard for the M3 wiring).                                                                       |

### Test Coverage

The new files (`AppSegmentedControlView`, `settings-tab-segmented-control`, the test files listed above) must ship with tests. The `frontend-coverage` CI job runs `pnpm test:coverage`; the 80% threshold is staged (currently commented out in `vitest.config.ts`) but the expectation is that new code does not regress coverage. The new test files in this spec are scoped to cover:

- The view wiring (refs/handlers attach to the right DOM nodes).
- The lifted hook usage (the parent passes the machine handler to the chevron).
- The CSS transition utility classes on the chevron.
- The stability of `handleTabChange` (no duplicate IPC).

---

## Acceptance Criteria

### Issue 1 (Animation)

- [ ] Clicking a tray page tab directly animates the active pill to the new tab (unchanged).
- [ ] Clicking the tray chevron (forward or backward) animates the active pill to the new tab.
- [ ] Clicking the tray chevron at the first or last tab (where `nextIndex === currentIndex`) does not change the selected tab and does not animate the pill.
- [ ] Hovering a tab, then selecting it via the chevron, still clears the hover indicator before the active pill animates.
- [ ] `SettingsTabSegmentedControl` continues to work unchanged in the settings form.

### Issue 2 (Crash)

- [ ] **Manual:** 20 rapid chevron clicks in succession do not crash the app. The window remains open and responsive. (Cannot be fully unit-tested; the automated test below covers "rapid clicks don't throw an error" in `tray-panel-tab-list.test.tsx`.)
- [ ] The `use-gsap-overflow-visibility` folder and any colocated test are deleted; no source file imports from it.
- [ ] No `useGSAP`, `gsap`, or `@gsap/react` import remains in `tray-tab-chevron.tsx` or `scroll-fade-overlays.tsx`.
- [ ] **Manual:** The chevron fade animation is visually equivalent (0.2s opacity + 4px translate) to the previous GSAP version.
- [ ] With `prefers-reduced-motion: reduce` enabled, the chevron snaps (no transition).

### Issue 3 (Stability)

- [ ] `handleTabChange` is referentially stable when `settings` is unchanged.
- [ ] `syncTrayUsage` is called once per `selectedTab` change (via the effect), not twice.
- [ ] Rapid chevron clicks no longer queue duplicate `syncTrayUsage` IPC calls.

### Automated

- [ ] `pnpm lint` passes.
- [ ] `pnpm format:check` passes.
- [ ] `pnpm test` passes with all new and existing tests.
- [ ] `pnpm build` passes.
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passes.
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` passes.
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` passes.

### Manual

- [ ] Open the tray panel. Click the right chevron rapidly 20 times. The window stays open, the pill animates to each tab, and the final selected tab matches the last click.
- [ ] Open the widget window. Repeat the rapid-chevron test. Same result.
- [ ] Click a tab directly — pill animates as before.
- [ ] Hover a tab, then click the chevron to navigate to that tab — hover clears, pill animates.
- [ ] With OS reduced-motion enabled, chevron and pill snap (no animation).
- [ ] Settings form: General/Providers tabs still switch and animate correctly.

---

## Non-Goals

- Do not change the chevron's visual design (icon, color, size, position).
- Do not change the active pill's visual design.
- Do not change the segmented control for inline or settings usage beyond what's required to make `AppSegmentedControl` a thin wrapper.
- Do not introduce a time-based click debounce on the chevron. CSS transitions remove the unnecessary GSAP reconstruction path without adding a UX trade-off.
- Do not change the cycle math (which tab to advance to, when to scroll vs change tab).
- Do not migrate `useTrayUiStore` or `useTrayPanelState` to a different state management library.
- Do not change the Rust commands or IPC protocol.

---

## Implementation Order

Per the TDD skill: for each issue, **write the failing test first**, then the implementation, then verify the test passes. The order of issues is:

1. **Issue 3 first** (memoize `handleTabChange` + remove duplicate IPC). Smallest diff, validates that the test suite can detect the change. No behavior risk.
2. **Issue 2 second** (CSS transitions for chevrons). Pure visual change, easy to roll back if needed. Removes repeated GSAP reconstruction before doing the larger refactor.
3. **Issue 1 third** (M3 — lift the state machine + move `SettingsTabSegmentedControl` to settings feature). Largest refactor. Built on a CSS-driven chevron and a memoized parent callback.

---

## Risks

- **M3 refactor surface**: `useAppSegmentControlState`'s return type becomes part of the public API of `app-segmented-control-view.tsx`. Future changes to the hook must keep the type stable, or update the view and its test.
- **CSS easing**: `ease-out` is close to GSAP's `power2.out` but not identical. For a 0.2s chevron fade, the difference is imperceptible. If pixel-perfect matching matters, we revisit.
- **A11y behavior change**: dropping `invisible` (`visibility: hidden`) means screen readers may announce the button if focus reaches it before opacity hits 0. `aria-hidden={!visible}` on the column and the button's `tabIndex={-1}` when hidden mitigate this. Verify with a screen reader if a11y is a concern.
- **`use-gsap-overflow-visibility` deletion**: verified by grep that only the two chevron files import it. If a future caller needs it, restore from git history.
- **`SettingsTabSegmentedControl` move**: verified by grep that only `settings-form.tsx` imports it. The move is a straight rename of the import path.
- **`pillReady` state ownership**: when `useAppSegmentControlState` is created at the parent (`TrayPanelTabList`) instead of inside the segmented control, `pillReady` lives in the parent. The view's `blockInteractionUntilPlaced` gate reads the same state value, so behavior is identical. The only difference is that the state now survives if the parent re-renders for an unrelated reason — which is the same as before because the parent (`TrayPanelTabList`) is the only consumer.
- **Test coverage**: the new files (`AppSegmentedControlView`, `settings-tab-segmented-control`, new test files) must ship with tests to keep `pnpm test:coverage` trending up. The 80% threshold is staged but the expectation per `AGENTS.md` is no regression.
- **Cross-platform**: all changes are pure JS/TS/CSS — no platform-specific code is introduced. CSS `transition` and `motion-reduce:` work identically on macOS, Windows, and Linux WebKit/WebView2/WebKitGTK. The state machine refactor is platform-agnostic.
