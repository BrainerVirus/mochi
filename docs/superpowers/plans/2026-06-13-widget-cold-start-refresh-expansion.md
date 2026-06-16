# Cold-Start Refresh Expansion — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expand the cold-start refresh trigger so the widget (and main window) auto-refresh stale data on boot, not just providers that have never been fetched.

**Architecture:** The `shouldRefreshEnabledProvidersOnBoot` function currently only checks `kind === "fetching"`. Expand to also check `"stale_error"`, `"error"`, and time-based `"fresh"` — where fresh data older than `refresh_interval_seconds` triggers a refresh. The function is pure (no side effects), so it's straightforward to test.

**Tech Stack:** TypeScript, Vitest

---

### Task 1: Update tests first (TDD — write failing tests)

**Files:**

- Modify: `src/hooks/use-cold-start-provider-refresh.test.ts`

- [ ] **Step 1: Update the `state()` helper to accept an optional `updatedAt` parameter**

Current helper:

```typescript
function state(
  provider: ProviderUsageState["provider"],
  kind: ProviderUsageState["kind"] = "fetching",
) {
  return {
    provider,
    kind,
    snapshot: null,
    health: kind === "fresh" ? "ok" : "stale",
    message: kind === "fetching" ? "fetching usage" : null,
    updated_at: "2026-05-31T12:00:00Z",
  } satisfies ProviderUsageState;
}
```

Replace with:

```typescript
function state(
  provider: ProviderUsageState["provider"],
  kind: ProviderUsageState["kind"] = "fetching",
  updatedAt?: string,
) {
  return {
    provider,
    kind,
    snapshot: null,
    health: kind === "fresh" ? "ok" : "stale",
    message: kind === "fetching" ? "fetching usage" : null,
    updated_at: updatedAt ?? "2026-05-31T12:00:00Z",
  } satisfies ProviderUsageState;
}
```

- [ ] **Step 2: Add new test cases for `shouldRefreshEnabledProvidersOnBoot`**

Replace the existing `describe("shouldRefreshEnabledProvidersOnBoot", ...)` block (lines 60-73) with:

```typescript
describe("shouldRefreshEnabledProvidersOnBoot", () => {
  it("refreshes when an enabled provider is in fetching state", () => {
    expect(shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [state("codex")])).toBe(true);
  });

  it("refreshes when an enabled provider has stale_error", () => {
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [state("codex", "stale_error")]),
    ).toBe(true);
  });

  it("refreshes when an enabled provider has error", () => {
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [state("codex", "error")]),
    ).toBe(true);
  });

  it("does not refresh when no providers are enabled", () => {
    expect(shouldRefreshEnabledProvidersOnBoot(settings([]), [state("codex")])).toBe(false);
  });

  it("does not refresh when all enabled providers have fresh data within refresh interval", () => {
    const recent = new Date().toISOString();
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [state("codex", "fresh", recent)]),
    ).toBe(false);
  });

  it("refreshes when fresh data is older than refresh interval", () => {
    const oldDate = new Date(Date.now() - 301_000).toISOString(); // 301s ago, threshold is 300s
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [state("codex", "fresh", oldDate)]),
    ).toBe(true);
  });

  it("does not refresh for missing credentials", () => {
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [
        state("codex", "missing_credentials"),
      ]),
    ).toBe(false);
  });

  it("does not refresh for credentials_need_refresh", () => {
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [
        state("codex", "credentials_need_refresh"),
      ]),
    ).toBe(false);
  });

  it("refreshes if ANY enabled provider needs refresh, even if others are fresh", () => {
    const recent = new Date().toISOString();
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex", "cursor"]), [
        state("codex", "stale_error"),
        state("cursor", "fresh", recent),
      ]),
    ).toBe(true);
  });
});
```

- [ ] **Step 3: Run the tests to see them fail**

```bash
pnpm test -- src/hooks/use-cold-start-provider-refresh.test.ts
```

Expected: the new tests fail because `shouldRefreshEnabledProvidersOnBoot` still only checks `kind === "fetching"`.

- [ ] **Step 4: Commit**

```bash
git add src/hooks/use-cold-start-provider-refresh.test.ts
git commit -m "test(cold-start): add failing tests for expanded refresh triggers"
```

---

### Task 2: Implement expanded logic

**Files:**

- Modify: `src/hooks/use-cold-start-provider-refresh.ts`

- [ ] **Step 1: Update `shouldRefreshEnabledProvidersOnBoot`**

Replace the function (lines 13-23) with:

```typescript
export function shouldRefreshEnabledProvidersOnBoot(
  settings: MochiSettings,
  states: ProviderUsageState[],
): boolean {
  if (settings.enabled_providers.length === 0) return false;

  const enabled = new Set(settings.enabled_providers);
  const refreshIntervalSeconds = settings.refresh_interval_seconds;

  return states.some((state) => {
    if (!enabled.has(state.provider)) return false;

    // Always retry providers that need a fetch
    if (state.kind === "fetching") return true;
    if (state.kind === "stale_error") return true;
    if (state.kind === "error") return true;

    // Time-based: refresh "fresh" data if it's older than the configured interval
    if (state.kind === "fresh" && state.updated_at) {
      const ageMs = Date.now() - new Date(state.updated_at).getTime();
      const thresholdMs = refreshIntervalSeconds * 1000;
      return ageMs > thresholdMs;
    }

    return false;
  });
}
```

- [ ] **Step 2: Run the tests to see them pass**

```bash
pnpm test -- src/hooks/use-cold-start-provider-refresh.test.ts
```

Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/hooks/use-cold-start-provider-refresh.ts
git commit -m "feat(cold-start): expand refresh triggers for stale_error, error, and time-based fresh"
```

---

### Task 3: Run full validation

**Files:**

- Project root

- [ ] **Step 1: Run full test suite**

```bash
pnpm test
```

Expected: all tests pass.

- [ ] **Step 2: Run lint**

```bash
pnpm lint
```

Expected: no lint errors.

- [ ] **Step 3: Run type check**

```bash
pnpm build
```

Expected: builds without errors.

- [ ] **Step 4: Commit**

```bash
git commit -m "chore: verify cold-start refresh expansion passes all checks"
```
