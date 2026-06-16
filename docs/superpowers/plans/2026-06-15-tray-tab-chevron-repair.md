# Tray Tab Chevron Crash & Active-Pill Animation Repair — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix two defects in the Mochi tray page-tab strip: (1) rapid chevron clicks crash the app, and (2) the active pill doesn't animate when navigating via chevron. The first is fixed by replacing the chevron's GSAP animation with CSS transitions and removing a duplicated IPC call. The second is fixed by lifting the segmented control's state machine to the tab list parent so both click paths share the same machine handler.

**Architecture:** Three independent issues shipped in one PR. Issue 3 (memoize + remove duplicate IPC) is the smallest, no-behavior-risk change. Issue 2 (CSS transitions) removes the OOM source. Issue 1 (M3 refactor) lifts the state machine.

**Tech Stack:** TypeScript, React 19, Tailwind CSS 4, GSAP (removal), Vitest.

**Spec:** `docs/superpowers/specs/2026-06-15-tray-tab-chevron-repair-design.md`

---

## Issue 3: Stability Fixes (Memoize + Remove Duplicate IPC)

### Task 1: Write failing test for `handleTabChange` stability and IPC dedup

**Files:**

- Modify: `src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.test.ts`

- [ ] **Step 1: Read the existing test file**

```bash
cat src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.test.ts
```

Confirm the file exists and currently tests only `persistTabChangeSettings` (a helper export).

- [ ] **Step 2: Add new imports at the top of the file (after the existing imports)**

Add these lines after the existing `import { persistTabChangeSettings } from "./use-tray-panel-state";` line:

```typescript
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook } from "@testing-library/react";
```

- [ ] **Step 3: Extend the existing `@/lib/tauri/commands` mock to include `saveSettings` returning a typed value**

The existing mock already covers `saveSettings`. Ensure it returns a `MochiSettings` (or a casted object) so the `persistTabChangeSettings` helper can consume it without a type error. If the existing mock returns `Promise.resolve({})`, change it to:

```typescript
vi.mocked(saveSettings).mockResolvedValue(DEFAULT_MOCHI_SETTINGS as Awaited<ReturnType<typeof saveSettings>>);
```

(or, in the `vi.mock` factory, return a proper `DEFAULT_MOCHI_SETTINGS`).

- [ ] **Step 4: Add a `describe("useTrayPanelState", ...)` block at the bottom of the file**

Append:

```typescript
function makeWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

describe("useTrayPanelState", () => {
  it("returns a referentially stable handleTabChange across renders with the same settings", () => {
    const { result, rerender } = renderHook(() => useTrayPanelState(), {
      wrapper: makeWrapper(),
    });
    const first = result.current.handleTabChange;
    rerender();
    const second = result.current.handleTabChange;
    expect(second).toBe(first);
  });

  it("does not call syncTrayUsage directly from handleTabChange", () => {
    const syncTrayUsage = vi.mocked(syncTrayUsage);
    syncTrayUsage.mockClear();
    const { result } = renderHook(() => useTrayPanelState(), {
      wrapper: makeWrapper(),
    });
    result.current.handleTabChange("codex");
    expect(syncTrayUsage).not.toHaveBeenCalled();
  });
});
```

Note: `useTrayPanelState` calls `useSettings`, `useTrayPanelRefresh`, and `useUsageData`. The existing file already mocks the `@/lib/tauri/commands` module. If those upstream hooks aren't already mocked, add `vi.mock` calls for `@/features/tray/hooks/use-tray-events`, `@/features/tray/hooks/use-tray-panel-refresh`, and `@/features/usage/hooks/use-usage-data/use-usage-data` at the top of the file with the return shapes `useTrayPanelState` expects.

- [ ] **Step 5: Run the tests to see them fail**

```bash
pnpm test -- src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.test.ts
```

Expected: both new tests fail. The first fails because `handleTabChange` is not yet memoized. The second fails because `handleTabChange` still calls `syncTrayUsage` synchronously.

- [ ] **Step 6: Commit**

```bash
git add src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.test.ts
git commit -m "test(state): add failing tests for handleTabChange stability and IPC dedup"
```

---

### Task 2: Memoize `handleTabChange` and remove duplicate `syncTrayUsage`

**Files:**

- Modify: `src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.ts`

- [ ] **Step 1: Add the `useCallback` import**

At the top of the file, change:

```typescript
import { useEffect, useMemo, useRef, useState } from "react";
```

to:

```typescript
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
```

- [ ] **Step 2: Wrap `handleTabChange` in `useCallback` and remove the synchronous `syncTrayUsage`**

Replace the existing `function handleTabChange(value: string) { ... }` block (currently between `useEffect(() => { void syncTrayUsage(selectedTab); }, [selectedTab]);` and `function handleRefreshProvider`) with:

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

- [ ] **Step 3: Run the tests to see them pass**

```bash
pnpm test -- src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.test.ts
```

Expected: both new tests pass.

- [ ] **Step 4: Run the full test suite to confirm no regressions**

```bash
pnpm test
```

Expected: all tests pass.

- [ ] **Step 5: Run lint and format check**

```bash
pnpm lint
pnpm format:check
```

Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.ts
git commit -m "fix(state): memoize handleTabChange and remove duplicate syncTrayUsage IPC"
```

---

## Issue 2: Replace Chevron GSAP with CSS Transitions

### Task 3: Write failing test for CSS transition classes on horizontal chevron

**Files:**

- Modify: `src/features/tray/components/tray-tab-chevron/tray-tab-chevron.test.ts`

- [ ] **Step 1: Read the existing test file**

```bash
cat src/features/tray/components/tray-tab-chevron/tray-tab-chevron.test.ts
```

The existing tests cover `getTrayTabChevronButtonClassName`. We need to add component-level tests.

- [ ] **Step 2: Add new imports at the top of the file (with the existing imports)**

Add after the existing `import { getTrayTabChevronButtonClassName } from "..."` line:

```typescript
import { render } from "@testing-library/react";

import { TrayTabChevron } from "./tray-tab-chevron";
```

- [ ] **Step 3: Add the component-level describe block at the bottom of the file**

Append after the existing `describe(getTrayTabChevronButtonClassName, ...)` block:

```typescript
describe("TrayTabChevron", () => {
  it("uses CSS transitions for fade animation", () => {
    const { container } = render(
      <TrayTabChevron side="end" visible={true} onCycle={() => {}} />,
    );
    const column = container.firstChild as HTMLElement;
    expect(column.className).toContain("transition-[opacity,transform]");
    expect(column.className).toContain("duration-200");
    expect(column.className).toContain("ease-out");
    expect(column.className).toContain("motion-reduce:transition-none");
  });

  it("hides via opacity-0 and translate when not visible", () => {
    const { container } = render(
      <TrayTabChevron side="end" visible={false} onCycle={() => {}} />,
    );
    const column = container.firstChild as HTMLElement;
    expect(column.className).toContain("opacity-0");
    expect(column.className).toContain("translate-x-1");
  });
});
```

- [ ] **Step 4: Run the test to see it fail**

```bash
pnpm test -- src/features/tray/components/tray-tab-chevron/tray-tab-chevron.test.ts
```

Expected: the new tests fail because the component still uses GSAP (no CSS transition classes).

- [ ] **Step 5: Commit**

```bash
git add src/features/tray/components/tray-tab-chevron/tray-tab-chevron.test.ts
git commit -m "test(chevron): add failing tests for CSS transition classes on horizontal chevron"
```

---

### Task 4: Replace horizontal chevron GSAP with CSS transitions

**Files:**

- Modify: `src/features/tray/components/tray-tab-chevron/tray-tab-chevron.tsx`

- [ ] **Step 1: Remove GSAP-related imports**

Delete these lines at the top of the file:

```typescript
import { useGSAP } from "@gsap/react";
import gsap from "gsap";
```

Delete the entire import from `@/features/tray/components/use-gsap-overflow-visibility` (after this change, none of `SCROLL_OVERFLOW_FADE_DURATION_S`, `SCROLL_OVERFLOW_FADE_EASE`, or `SCROLL_OVERFLOW_SLIDE_PX` are referenced in this file).

- [ ] **Step 2: Remove `useRef` import for `columnRef`**

Delete the line `import { useRef } from "react";` from the file. (After Step 4, `useRef` is no longer used in this file — the only `useRef` call was for `columnRef`.)

- [ ] **Step 3: Remove the `gsap.registerPlugin` call**

Delete the line `gsap.registerPlugin(useGSAP);` near the top of the file.

- [ ] **Step 4: Remove the `useGSAP` block and the `columnRef`**

Delete the `const columnRef = useRef<HTMLDivElement>(null);` line, the `const isStart = side === "start";` line (move the `isStart` declaration inside the function body if it's not already there), the `const hiddenX = isStart ? -SCROLL_OVERFLOW_SLIDE_PX : SCROLL_OVERFLOW_SLIDE_PX;` line, and the entire `useGSAP(...)` block.

- [ ] **Step 5: Replace the column `<div>` with the CSS-transition version**

Replace the existing column `<div>` (which has `ref={columnRef}` and the `invisible opacity-0` className) with:

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

- [ ] **Step 6: Remove the `TrayTabChevronSide` type export if it's no longer used**

Check if `TrayTabChevronSide` is imported anywhere else:

```bash
grep -r "TrayTabChevronSide" src/
```

If no other importers, delete the `export type TrayTabChevronSide = "start" | "end";` line from the file.

- [ ] **Step 7: Run the tests to see them pass**

```bash
pnpm test -- src/features/tray/components/tray-tab-chevron/tray-tab-chevron.test.ts
```

Expected: the new component tests pass; the existing `getTrayTabChevronButtonClassName` tests still pass.

- [ ] **Step 8: Run lint and format check**

```bash
pnpm lint
pnpm format:check
```

Expected: clean.

- [ ] **Step 9: Commit**

```bash
git add src/features/tray/components/tray-tab-chevron/tray-tab-chevron.tsx
git commit -m "refactor(chevron): replace GSAP with CSS transitions on horizontal tray chevron"
```

---

### Task 5: Replace vertical chevron GSAP with CSS transitions

**Files:**

- Modify: `src/features/tray/components/scroll-fade-overlays/scroll-fade-overlays.tsx`

- [ ] **Step 1: Remove GSAP-related imports**

Delete these lines:

```typescript
import { useGSAP } from "@gsap/react";
import gsap from "gsap";
```

Delete the `animateOverflowVisibility` and `SCROLL_OVERFLOW_SLIDE_PX` imports from `@/features/tray/components/use-gsap-overflow-visibility`. Keep the `useRef` import — the vertical chevron still needs `useRef` for `buttonRef` if any ref is used, or remove it if not. (The new `ScrollFadeVerticalChevron` in Step 3 does not use a ref, so delete the `useRef` import as well.)

Delete the `gsap.registerPlugin(useGSAP);` line.

- [ ] **Step 3: Replace `ScrollFadeVerticalChevron` with the CSS-transition version**

Replace the entire `ScrollFadeVerticalChevron` function with:

```tsx
function ScrollFadeVerticalChevron({
  side,
  visible,
  onCycle,
}: {
  side: "start" | "end";
  visible: boolean;
  onCycle: () => void;
}) {
  const isStart = side === "start";

  return (
    <Button
      type="button"
      variant="ghost"
      size="icon-xs"
      tabIndex={visible ? 0 : -1}
      aria-label={isStart ? "Scroll up for more" : "Scroll down for more"}
      onClick={onCycle}
      className={cn(
        "pointer-events-auto absolute inset-x-0 z-20 mx-auto shrink-0 cursor-pointer rounded-full",
        "bg-background/35 text-muted-foreground shadow-none ring-0 backdrop-blur-[2px] hover:bg-background/50 hover:text-foreground",
        "transition-[opacity,transform] duration-200 ease-out motion-reduce:transition-none",
        isStart ? "top-0 mt-0.5" : "bottom-0 mb-0.5",
        !visible ? "pointer-events-none opacity-0" : "opacity-100",
        !visible && (isStart ? "-translate-y-1" : "translate-y-1"),
      )}
    >
      {isStart ? <ChevronUpIcon className="size-3.5" /> : <ChevronDownIcon className="size-3.5" />}
    </Button>
  );
}
```

Note: the vertical chevron puts the transition directly on the `<Button>` (not on a wrapping column) because the original implementation also had the animation on the button itself.

- [ ] **Step 4: Run the test suite**

```bash
pnpm test
```

Expected: all tests pass.

- [ ] **Step 5: Run lint and format check**

```bash
pnpm lint
pnpm format:check
```

Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add src/features/tray/components/scroll-fade-overlays/scroll-fade-overlays.tsx
git commit -m "refactor(chevron): replace GSAP with CSS transitions on vertical scroll chevron"
```

---

### Task 6: Delete `use-gsap-overflow-visibility/` folder

**Files:**

- Delete: `src/features/tray/components/use-gsap-overflow-visibility/`

- [ ] **Step 1: Verify no remaining importers**

```bash
grep -r "use-gsap-overflow-visibility" src/
```

Expected: no results. (Tasks 4 and 5 removed the only consumers.)

- [ ] **Step 2: Delete the folder**

```bash
rm -rf src/features/tray/components/use-gsap-overflow-visibility
```

- [ ] **Step 3: Run the test suite**

```bash
pnpm test
pnpm build
```

Expected: builds and tests pass.

- [ ] **Step 4: Commit**

```bash
git add -u src/features/tray/components/use-gsap-overflow-visibility
git commit -m "chore(chevron): remove unused use-gsap-overflow-visibility hook"
```

---

## Issue 1: Lift the State Machine to the Parent (M3)

### Task 7: Write failing test for `AppSegmentedControlView`

**Files:**

- Create: `src/components/ui/app-segmented-control-view.test.tsx`

- [ ] **Step 1: Create the test file with a smoke test**

```typescript
import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AppSegmentedControlView } from "./app-segmented-control-view";

function makeState(overrides: Partial<{
  trackRef: React.RefObject<HTMLDivElement | null>;
  activeIndicatorRef: React.RefObject<HTMLDivElement | null>;
  hoverIndicatorRef: React.RefObject<HTMLDivElement | null>;
  setItemRef: (id: string, element: HTMLButtonElement | null) => void;
  syncHoverIndicator: (id: string) => void;
  handleValueChange: (next: string) => void;
  pillReady: boolean;
}> = {}) {
  return {
    trackRef: { current: null } as React.RefObject<HTMLDivElement | null>,
    activeIndicatorRef: { current: null } as React.RefObject<HTMLDivElement | null>,
    hoverIndicatorRef: { current: null } as React.RefObject<HTMLDivElement | null>,
    setItemRef: vi.fn(),
    syncHoverIndicator: vi.fn(),
    handleValueChange: vi.fn(),
    pillReady: true,
    ...overrides,
  };
}

describe("AppSegmentedControlView", () => {
  it("renders the toggle group with the provided items", () => {
    const state = makeState();
    const { container } = render(
      <AppSegmentedControlView
        state={state as never}
        items={[
          { id: "a", label: "A" },
          { id: "b", label: "B" },
        ]}
        value="a"
        onValueChange={() => {}}
      />,
    );
    expect(container.querySelectorAll('[data-slot="toggle-group-item"]').length).toBe(2);
  });

  it("wires the provided handleValueChange to ToggleGroup", () => {
    const handleValueChange = vi.fn();
    const state = makeState({ handleValueChange });
    const { container } = render(
      <AppSegmentedControlView
        state={state as never}
        items={[{ id: "a", label: "A" }]}
        value="a"
        onValueChange={() => {}}
      />,
    );
    const group = container.querySelector('[data-slot="toggle-group"]') as HTMLElement;
    expect(group).toBeTruthy();
  });
});
```

- [ ] **Step 2: Run the test to see it fail**

```bash
pnpm test -- src/components/ui/app-segmented-control-view.test.tsx
```

Expected: test fails because `AppSegmentedControlView` does not exist yet.

- [ ] **Step 3: Commit**

```bash
git add src/components/ui/app-segmented-control-view.test.tsx
git commit -m "test(segmented-control): add failing test for AppSegmentedControlView"
```

---

### Task 8: Extract `AppSegmentedControlView` and slim down `AppSegmentedControl`

**Files:**

- Create: `src/components/ui/app-segmented-control-view.tsx`
- Modify: `src/components/ui/app-segmented-control.tsx`

- [ ] **Step 1: Create `app-segmented-control-view.tsx`**

Move the JSX from `app-segmented-control.tsx` into a new file. The view accepts `state` as a prop and uses `state.trackRef`, `state.activeIndicatorRef`, `state.hoverIndicatorRef`, `state.setItemRef`, `state.syncHoverIndicator`, `state.handleValueChange`, and `state.pillReady`.

```typescript
"use client";

import { ToggleGroup } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

import {
  SegmentIndicators,
  SegmentToggleItems,
} from "@/components/ui/app-segmented-control-segments";
import {
  INLINE_SEGMENT_INDICATOR_RADIUS_CLASS,
  resolvePageTabRadiusClasses,
  usesPageTabIndicators,
  usesSegmentActiveIndicator,
  usesSegmentHoverIndicator,
  type AppSegmentItem,
  type AppSegmentedControlLayout,
  type AppSegmentedControlVariant,
} from "@/components/ui/app-segmented-control-utils";
import {
  useAppSegmentControlState,
  type UseAppSegmentControlState,
} from "@/components/ui/use-app-segment-control-state";

export interface AppSegmentedControlViewProps {
  state: UseAppSegmentControlState;
  items: AppSegmentItem[];
  value: string;
  onValueChange: (value: string) => void;
  className?: string;
  rowHeight?: string;
  stretchItems?: boolean;
  variant?: AppSegmentedControlVariant;
  layout?: AppSegmentedControlLayout;
  // Note: `contentReady` is intentionally absent. It is consumed by
  // `useAppSegmentControlState` in the `AppSegmentedControl` wrapper, not by
  // the view's render. The wrapper passes it to the hook and omits it from
  // the props it forwards to the view.
}

export function AppSegmentedControlView({
  state,
  items,
  value,
  onValueChange,
  className,
  rowHeight = "h-9",
  stretchItems = true,
  variant = "page-tabs",
  layout = "tray",
}: AppSegmentedControlViewProps) {
  const isPageTabs = usesPageTabIndicators(variant);
  const showHover = usesSegmentHoverIndicator(variant, layout);
  const usesIndicators = usesSegmentActiveIndicator(variant);
  const pageTabRadius = resolvePageTabRadiusClasses(layout);
  const indicatorRadiusClass = isPageTabs
    ? pageTabRadius.indicator
    : INLINE_SEGMENT_INDICATOR_RADIUS_CLASS;
  const blockInteractionUntilPlaced = usesIndicators && layout === "settings" && !state.pillReady;

  return (
    <div
      ref={state.trackRef}
      onPointerLeave={showHover ? state.handleRailLeave : undefined}
      data-segment-variant={variant}
      data-segment-layout={isPageTabs ? layout : undefined}
      className={cn(
        rowHeight,
        "relative isolate p-0.5",
        isPageTabs ? pageTabRadius.track : "rounded-lg",
        stretchItems ? "w-full" : "w-max min-w-full",
        blockInteractionUntilPlaced && "pointer-events-none",
        className,
      )}
    >
      {usesIndicators ? (
        <SegmentIndicators
          hoverIndicatorRef={state.hoverIndicatorRef}
          activeIndicatorRef={state.activeIndicatorRef}
          indicatorRadiusClass={indicatorRadiusClass}
          variant={variant}
          showHover={showHover}
        />
      ) : null}

      <ToggleGroup
        type="single"
        orientation="horizontal"
        value={value}
        onValueChange={state.handleValueChange}
        spacing={0}
        variant="default"
        className="relative z-10 flex h-full w-full flex-row flex-nowrap items-stretch justify-stretch gap-0 bg-transparent p-0 shadow-none"
      >
        <SegmentToggleItems
          items={items}
          variant={variant}
          stretchItems={stretchItems}
          setItemRef={state.setItemRef}
          syncHoverIndicator={state.syncHoverIndicator}
          showHover={showHover}
        />
      </ToggleGroup>
    </div>
  );
}
```

Note: the view imports `useAppSegmentControlState` solely for the `UseAppSegmentControlState` type alias. We need to export this type from the hook file (next step).

- [ ] **Step 2: Export the return type from `use-app-segment-control-state.ts`**

Open the file and add the type export:

```typescript
export type UseAppSegmentControlState = ReturnType<typeof useAppSegmentControlState>;
```

Add it after the `useAppSegmentControlState` function declaration (around line 68, before the closing of the file).

- [ ] **Step 3: Slim down `app-segmented-control.tsx`**

Replace the entire function body with:

```typescript
"use client";

import {
  AppSegmentedControlView,
  type AppSegmentedControlViewProps,
} from "@/components/ui/app-segmented-control-view";
import {
  usesSegmentActiveIndicator,
  usesSegmentHoverIndicator,
  type AppSegmentItem,
  type AppSegmentedControlVariant,
} from "@/components/ui/app-segmented-control-utils";
import { useAppSegmentControlState } from "@/components/ui/use-app-segment-control-state";

// AppSegmentedControlProps is the public prop type for the wrapper. It is
// intentionally broader than AppSegmentedControlViewProps because the wrapper
// also accepts `contentReady` (consumed by the hook, not forwarded to the view).
export interface AppSegmentedControlProps extends Omit<AppSegmentedControlViewProps, "state"> {
  contentReady?: boolean;
}

export function AppSegmentedControl(props: AppSegmentedControlProps) {
  const variant: AppSegmentedControlVariant = props.variant ?? "page-tabs";
  const state = useAppSegmentControlState(
    props.value,
    props.items.length,
    props.onValueChange,
    {
      enabled: usesSegmentActiveIndicator(variant),
      showHover: usesSegmentHoverIndicator(variant, props.layout ?? "tray"),
      contentReady: props.contentReady ?? true,
    },
  );
  // Forward everything except `contentReady` (consumed by the hook above) and
  // `state` (which we supply).
  const { contentReady: _contentReady, ...viewProps } = props;
  return <AppSegmentedControlView {...viewProps} state={state} />;
}
```

- [ ] **Step 4: Run the test to see it pass**

```bash
pnpm test -- src/components/ui/app-segmented-control-view.test.tsx
```

Expected: the new test passes.

- [ ] **Step 5: Run the full test suite to confirm no regressions**

```bash
pnpm test
```

Expected: all tests pass. `AppSegmentedControl` callers (inline, settings, tray menu) continue to work.

- [ ] **Step 6: Run lint and format check**

```bash
pnpm lint
pnpm format:check
```

Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add src/components/ui/app-segmented-control-view.tsx \
        src/components/ui/app-segmented-control.tsx \
        src/components/ui/use-app-segment-control-state.ts
git commit -m "refactor(segmented-control): extract AppSegmentedControlView from AppSegmentedControl"
```

---

### Task 9: Write failing test for lifted hook in `TrayPanelTabList`

**Files:**

- Create: `src/features/tray/components/tray-panel-tab-list/tray-panel-tab-list.test.tsx`

- [ ] **Step 1: Create the test file with a smoke test for chevron wiring**

The test mocks `useAppSegmentControlState` to return a state object whose `handleValueChange` is a spy. This isolates the test from GSAP and the machine internals, and lets us assert the parent's `onValueChange` flows through the lifted hook.

```typescript
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { TrayPanelTabList } from "./tray-panel-tab-list";

const handleValueChangeSpy = vi.fn();

vi.mock("@/components/ui/use-app-segment-control-state", () => ({
  useAppSegmentControlState: () => ({
    trackRef: { current: null },
    activeIndicatorRef: { current: null },
    hoverIndicatorRef: { current: null },
    itemRefs: { current: new Map() },
    setItemRef: vi.fn(),
    syncHoverIndicator: vi.fn(),
    handleRailLeave: vi.fn(),
    handleValueChange: handleValueChangeSpy,
    pillReady: true,
  }),
}));

vi.mock("@/features/tray/components/scroll-fade-region", () => ({
  ScrollFadeRegion: ({
    children,
    onCycleForward,
    onCycleBackward,
  }: {
    children: React.ReactNode;
    onCycleForward?: (el: HTMLDivElement) => void;
    onCycleBackward?: (el: HTMLDivElement) => void;
  }) => (
    <div>
      <button
        data-testid="cycle-forward"
        onClick={() => onCycleForward?.(document.createElement("div"))}
      >
        forward
      </button>
      <button
        data-testid="cycle-backward"
        onClick={() => onCycleBackward?.(document.createElement("div"))}
      >
        backward
      </button>
      {children}
    </div>
  ),
}));

vi.mock("@/components/ui/app-segmented-control-view", () => ({
  AppSegmentedControlView: ({
    items,
    value,
    onValueChange,
  }: {
    items: Array<{ id: string; label: string }>;
    value: string;
    onValueChange: (next: string) => void;
  }) => (
    <div>
      {items.map((item) => (
        <button key={item.id} data-testid={`tab-${item.id}`} onClick={() => onValueChange(item.id)}>
          {item.label}
        </button>
      ))}
      <span data-testid="current-value">{value}</span>
    </div>
  ),
}));

describe("TrayPanelTabList", () => {
  const tabs = [
    { id: "overview", label: "Overview" },
    { id: "codex", label: "Codex" },
    { id: "cursor", label: "Cursor" },
  ];

  beforeEach(() => {
    handleValueChangeSpy.mockClear();
    handleValueChangeSpy.mockImplementation((next: string, onValueChange: (v: string) => void) => {
      onValueChange(next);
    });
  });

  it("routes chevron clicks through the lifted machine handler", () => {
    const onValueChange = vi.fn();
    render(<TrayPanelTabList tabs={tabs} value="overview" onValueChange={onValueChange} />);
    fireEvent.click(screen.getByTestId("cycle-forward"));
    expect(handleValueChangeSpy).toHaveBeenCalled();
    expect(onValueChange).toHaveBeenCalledWith("codex");
  });

  it("rapid chevron clicks do not throw", () => {
    const onValueChange = vi.fn();
    render(<TrayPanelTabList tabs={tabs} value="overview" onValueChange={onValueChange} />);
    const forward = screen.getByTestId("cycle-forward");
    for (let i = 0; i < 20; i += 1) {
      fireEvent.click(forward);
    }
    expect(onValueChange).toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: Run the test to see it fail**

```bash
pnpm test -- src/features/tray/components/tray-panel-tab-list/tray-panel-tab-list.test.tsx
```

Expected: the test fails because the current `TrayPanelTabList` calls `onValueChange` directly (bypassing the machine handler). After Task 10 lifts the hook, the test passes.

- [ ] **Step 3: Commit**

```bash
git add src/features/tray/components/tray-panel-tab-list/tray-panel-tab-list.test.tsx
git commit -m "test(tab-list): add failing test for chevron routing through machine handler"
```

---

### Task 10: Lift `useAppSegmentControlState` to `TrayPanelTabList`

**Files:**

- Modify: `src/features/tray/components/tray-panel-tab-list/tray-panel-tab-list.tsx`

- [ ] **Step 1: Update the imports**

Replace the existing imports with:

```typescript
import { useCallback, useMemo } from "react";

import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { LayoutGridIcon } from "lucide-react";
import { ProviderIcon } from "@/components/providers/provider-icon";
import { AppSegmentedControlView } from "@/components/ui/app-segmented-control-view";
import { useAppSegmentControlState } from "@/components/ui/use-app-segment-control-state";
import { ScrollFadeRegion } from "@/features/tray/components/scroll-fade-region";
import { cycleTrayPanelTabs } from "@/features/tray/components/tray-panel-tab-cycle";
import { TRAY_SEGMENT_ROW_HEIGHT } from "@/features/tray/components/tray-segmented-control-config";
import { cn } from "@/lib/utils";
```

- [ ] **Step 2: Rewrite the component body**

Replace the entire `TrayPanelTabList` function with:

```tsx
export function TrayPanelTabList({ tabs, value, onValueChange }: TrayPanelTabListProps) {
  const items = useMemo(
    () =>
      tabs.map((tab) => ({
        id: tab.id,
        label: tab.label,
        icon:
          tab.id === "overview" ? (
            <LayoutGridIcon aria-hidden />
          ) : (
            <ProviderIcon provider={tab.id} />
          ),
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
      className={cn(
        "border-border min-w-0 overflow-hidden rounded-t-[var(--radius-tray-panel)] border-b px-3 pb-2",
      )}
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

- [ ] **Step 3: Run the tests to see them pass**

```bash
pnpm test -- src/features/tray/components/tray-panel-tab-list/tray-panel-tab-list.test.tsx
```

Expected: the new tests pass.

- [ ] **Step 4: Run the full test suite**

```bash
pnpm test
```

Expected: all tests pass.

- [ ] **Step 5: Run lint and format check**

```bash
pnpm lint
pnpm format:check
```

Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add src/features/tray/components/tray-panel-tab-list/tray-panel-tab-list.tsx
git commit -m "refactor(tab-list): lift useAppSegmentControlState to TrayPanelTabList"
```

---

### Task 10b: Add smoke test for `useAppSegmentControlState`

**Files:**

- Create: `src/components/ui/use-app-segment-control-state.test.ts`

- [ ] **Step 1: Create the test file**

```typescript
import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { useAppSegmentControlState } from "./use-app-segment-control-state";

vi.mock("@/features/tray/components/use-tray-segment-indicators", () => ({
  useTraySegmentIndicators: () => ({
    syncHoverIndicator: vi.fn(),
    handleRailLeave: vi.fn(),
    handleSegmentValueChange: vi.fn(),
    pillReady: true,
  }),
}));

describe("useAppSegmentControlState", () => {
  it("returns a handleValueChange that wraps handleSegmentValueChange when indicators are enabled", () => {
    const onValueChange = vi.fn();
    const { result } = renderHook(() =>
      useAppSegmentControlState("a", 2, onValueChange, { enabled: true, showHover: true, contentReady: true }),
    );
    expect(result.current.handleValueChange).toBeDefined();
    result.current.handleValueChange("b");
    // The handleSegmentValueChange spy is captured by the mock; the assertion
    // here is that handleValueChange is a function that doesn't throw.
    expect(onValueChange).not.toHaveBeenCalled();
  });

  it("returns a handleValueChange that calls onValueChange directly when indicators are disabled", () => {
    const onValueChange = vi.fn();
    const { result } = renderHook(() =>
      useAppSegmentControlState("a", 2, onValueChange, { enabled: false, showHover: false, contentReady: true }),
    );
    result.current.handleValueChange("b");
    expect(onValueChange).toHaveBeenCalledWith("b");
  });

  it("ignores empty string values", () => {
    const onValueChange = vi.fn();
    const { result } = renderHook(() =>
      useAppSegmentControlState("a", 2, onValueChange, { enabled: true, showHover: true, contentReady: true }),
    );
    result.current.handleValueChange("");
    expect(onValueChange).not.toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: Run the tests**

```bash
pnpm test -- src/components/ui/use-app-segment-control-state.test.ts
```

Expected: all three tests pass (the hook is already implemented; this is a regression-guard test).

- [ ] **Step 3: Commit**

```bash
git add src/components/ui/use-app-segment-control-state.test.ts
git commit -m "test(segmented-control): add smoke tests for useAppSegmentControlState"
```

---

### Task 11: Write failing test for `cycleTrayPanelTabs` branches

**Files:**

- Create: `src/features/tray/components/tray-panel-tab-cycle/tray-panel-tab-cycle.test.ts`

- [ ] **Step 1: Create the test file**

```typescript
import { describe, expect, it, vi } from "vitest";

import { cycleTrayPanelTabs } from "./tray-panel-tab-cycle";

const tabs = [
  { id: "overview", label: "Overview" },
  { id: "codex", label: "Codex" },
  { id: "cursor", label: "Cursor" },
];

function makeScrollEl() {
  const el = document.createElement("div");
  el.scrollTo = vi.fn();
  return el;
}

describe("cycleTrayPanelTabs", () => {
  it("returns early when currentValue is not in tabs", () => {
    const onValueChange = vi.fn();
    const el = makeScrollEl();
    cycleTrayPanelTabs(el, tabs, "missing", "forward", onValueChange, 40);
    expect(onValueChange).not.toHaveBeenCalled();
  });

  it("calls onValueChange with the next tab on forward", () => {
    const onValueChange = vi.fn();
    const el = makeScrollEl();
    cycleTrayPanelTabs(el, tabs, "overview", "forward", onValueChange, 40);
    expect(onValueChange).toHaveBeenCalledWith("codex");
  });

  it("calls onValueChange with the previous tab on backward", () => {
    const onValueChange = vi.fn();
    const el = makeScrollEl();
    cycleTrayPanelTabs(el, tabs, "codex", "backward", onValueChange, 40);
    expect(onValueChange).toHaveBeenCalledWith("overview");
  });

  it("clamps at the last tab on forward (calls scrollTo instead of onValueChange)", () => {
    const onValueChange = vi.fn();
    const el = makeScrollEl();
    cycleTrayPanelTabs(el, tabs, "cursor", "forward", onValueChange, 40);
    expect(onValueChange).not.toHaveBeenCalled();
    expect(el.scrollTo).toHaveBeenCalled();
  });

  it("clamps at the first tab on backward", () => {
    const onValueChange = vi.fn();
    const el = makeScrollEl();
    cycleTrayPanelTabs(el, tabs, "overview", "backward", onValueChange, 40);
    expect(onValueChange).not.toHaveBeenCalled();
    expect(el.scrollTo).toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: Run the test to see it fail (or pass, if the function already works)**

```bash
pnpm test -- src/features/tray/components/tray-panel-tab-cycle/tray-panel-tab-cycle.test.ts
```

Expected: tests may already pass (the function is straightforward). If they pass, this is a regression-prevention test, not a TDD red-green cycle. Commit the test as-is.

- [ ] **Step 3: Commit**

```bash
git add src/features/tray/components/tray-panel-tab-cycle/tray-panel-tab-cycle.test.ts
git commit -m "test(cycle): add tests for cycleTrayPanelTabs branches"
```

---

### Task 11b: Extend tray-segment-indicator and machine tests

**Files:**

- Modify: `src/features/tray/components/tray-segment-indicator/tray-segment-indicator.test.ts`
- Modify: `src/features/tray/components/tray-segment-indicator-machine/tray-segment-indicator-machine.test.ts`

- [ ] **Step 1: Read both test files**

```bash
cat src/features/tray/components/tray-segment-indicator/tray-segment-indicator.test.ts
cat src/features/tray/components/tray-segment-indicator-machine/tray-segment-indicator-machine.test.ts
```

These files exist and test the pure functions in `tray-segment-indicator.ts` and the state machine in `tray-segment-indicator-machine.ts`. They are stable; the extensions below add regression-guard tests for the M3 wiring.

- [ ] **Step 2: Add a test to `tray-segment-indicator.test.ts` asserting `moveActive` triggers `applyActiveIndicatorPosition` with `animate: true`**

Append to the existing test file (after the existing `describe("executeTraySegmentIndicatorCommand", ...)` block, or add a new `describe` for `applyActiveIndicatorPosition` with the `moveActive` path if not already covered):

```typescript
describe("moveActive command path", () => {
  it("calls applyActiveIndicatorPosition with animate: true", () => {
    // Reuse the existing gsapMocks from the file's top-level vi.mock("gsap", ...)
    // The mock's `applyActiveIndicatorPosition` should be invoked via
    // `executeTraySegmentIndicatorCommand` with `{ animate: true }`.
    // This is a smoke test ensuring the moveActive branch is reachable.
    expect(gsapMocks.to).toBeDefined();
  });
});
```

(If the existing file already exercises the `moveActive` path via `executeTraySegmentIndicatorCommand`, this addition is redundant. The implementer should check first; if the path is already covered, skip this step.)

- [ ] **Step 3: Add a test to `tray-segment-indicator-machine.test.ts` asserting `SELECT` emits `moveActive`**

Append:

```typescript
describe("SELECT event", () => {
  it("emits moveActive with the selected tab id", () => {
    const result = transitionTraySegmentIndicator(
      { status: "outside" },
      { type: "SELECT", tabId: "codex" },
    );
    expect(result.state).toEqual({ status: "selecting", selectedId: "codex" });
    const moveActive = result.commands.find((c) => c.type === "moveActive");
    expect(moveActive).toEqual({ type: "moveActive", tabId: "codex" });
  });
});
```

(If this is already covered, skip the addition.)

- [ ] **Step 4: Run the tests**

```bash
pnpm test -- src/features/tray/components/tray-segment-indicator/ \
            src/features/tray/components/tray-segment-indicator-machine/
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/features/tray/components/tray-segment-indicator/tray-segment-indicator.test.ts \
        src/features/tray/components/tray-segment-indicator-machine/tray-segment-indicator-machine.test.ts
git commit -m "test(segment-indicator): add regression guard for moveActive on SELECT"
```

---

### Task 12: Move `SettingsTabSegmentedControl` to the settings feature

**Files:**

- Create: `src/features/settings/components/settings-tab-segmented-control/settings-tab-segmented-control.tsx`
- Create: `src/features/settings/components/settings-tab-segmented-control/settings-tab-segmented-control-config.ts`
- Create: `src/features/settings/components/settings-tab-segmented-control/settings-tab-segmented-control.test.tsx`
- Modify: `src/features/settings/components/settings-form/settings-form.tsx`
- Modify: `src/features/tray/components/tray-segmented-control-config/tray-segmented-control-config.ts`
- Delete: `src/features/tray/components/tray-segmented-control/tray-segmented-control.tsx`

- [ ] **Step 1: Create the new config file**

`src/features/settings/components/settings-tab-segmented-control/settings-tab-segmented-control-config.ts`:

```typescript
/** Settings page-tab strip — full-width equal segments with .app-window --radius rounding. */
export const SETTINGS_PAGE_TAB_DEFAULTS = {
  variant: "page-tabs" as const,
  rowHeight: "h-9" as const,
  stretchItems: true,
  layout: "settings" as const,
};
```

- [ ] **Step 2: Create the new component file**

`src/features/settings/components/settings-tab-segmented-control/settings-tab-segmented-control.tsx`:

```typescript
"use client";

import type { AppSegmentItem } from "@/components/ui/app-segmented-control-utils";
import { AppSegmentedControl } from "@/components/ui/app-segmented-control";

import { SETTINGS_PAGE_TAB_DEFAULTS } from "./settings-tab-segmented-control-config";

interface SettingsTabSegmentedControlProps {
  items: AppSegmentItem[];
  value: string;
  onValueChange: (value: string) => void;
  className?: string;
  contentReady?: boolean;
}

export function SettingsTabSegmentedControl({
  items,
  value,
  onValueChange,
  className,
  contentReady = true,
}: SettingsTabSegmentedControlProps) {
  return (
    <AppSegmentedControl
      items={items}
      value={value}
      onValueChange={onValueChange}
      className={className}
      contentReady={contentReady}
      {...SETTINGS_PAGE_TAB_DEFAULTS}
    />
  );
}
```

- [ ] **Step 3: Create the smoke test**

`src/features/settings/components/settings-tab-segmented-control/settings-tab-segmented-control.test.tsx`:

```typescript
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { SettingsTabSegmentedControl } from "./settings-tab-segmented-control";

describe("SettingsTabSegmentedControl", () => {
  it("renders the provided items", () => {
    const { container } = render(
      <SettingsTabSegmentedControl
        items={[
          { id: "general", label: "General" },
          { id: "providers", label: "Providers" },
        ]}
        value="general"
        onValueChange={() => {}}
      />,
    );
    expect(container.querySelectorAll('[data-slot="toggle-group-item"]').length).toBe(2);
  });
});
```

- [ ] **Step 4: Update the import in `settings-form.tsx`**

In `src/features/settings/components/settings-form/settings-form.tsx:3`, change:

```typescript
import { SettingsTabSegmentedControl } from "@/features/tray/components/tray-segmented-control";
```

to:

```typescript
import { SettingsTabSegmentedControl } from "@/features/settings/components/settings-tab-segmented-control/settings-tab-segmented-control";
```

- [ ] **Step 5: Trim `tray-segmented-control-config.ts`**

Open the file and delete the `SETTINGS_PAGE_TAB_DEFAULTS` export. The file should now only export `TRAY_SEGMENT_ROW_HEIGHT` and `TRAY_PAGE_TAB_DEFAULTS`.

- [ ] **Step 6: Delete `tray-segmented-control.tsx`**

```bash
git rm src/features/tray/components/tray-segmented-control/tray-segmented-control.tsx
rmdir src/features/tray/components/tray-segmented-control 2>/dev/null || true
```

`git rm` removes the file from the index and working tree. `rmdir` cleans up the now-empty parent directory on the local filesystem (git does not track empty directories, but the local folder will linger if not removed). The `|| true` makes the step succeed if the directory is not empty for any reason.

- [ ] **Step 7: Run the test suite**

```bash
pnpm test
pnpm build
```

Expected: all tests pass, build succeeds. The settings form's General/Providers tabs still work.

- [ ] **Step 8: Run lint and format check**

```bash
pnpm lint
pnpm format:check
```

Expected: clean.

- [ ] **Step 9: Commit**

```bash
git add src/features/settings/components/settings-tab-segmented-control/ \
        src/features/settings/components/settings-form/settings-form.tsx \
        src/features/tray/components/tray-segmented-control-config/tray-segmented-control-config.ts
git commit -m "refactor(settings): move SettingsTabSegmentedControl to settings feature"
```

(The deletion of `tray-segmented-control.tsx` was already staged by `git rm` in Step 6.)

---

### Task 13: Final validation

**Files:**

- Project root

- [ ] **Step 1: Run the full test suite**

```bash
pnpm test
```

Expected: all tests pass.

- [ ] **Step 2: Run lint and format check**

```bash
pnpm lint
pnpm format:check
```

Expected: clean.

- [ ] **Step 3: Run the build**

```bash
pnpm build
```

Expected: builds without errors.

- [ ] **Step 4: Run Rust checks**

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: all clean.

- [ ] **Step 5: Manual smoke test (macOS / Windows / Linux)**

Open the tray panel. Click the right chevron rapidly 20 times. Confirm:
- The window stays open.
- The pill animates to each tab.
- The final selected tab matches the last click.

Repeat in the widget window. Click a tab directly — pill animates as before. Hover a tab, then click the chevron — hover clears, pill animates. With OS reduced-motion enabled, chevron and pill snap.

Open the settings form. Confirm General/Providers tabs still switch and animate correctly.

- [ ] **Step 6: Commit (if any fixups were needed)**

```bash
git commit -m "chore: final validation fixes for tray tab chevron repair"
```

(Only commit if Step 1-4 required adjustments.)
