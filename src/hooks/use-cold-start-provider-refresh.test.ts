import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  syncCurrentTrayUsage,
  useTrayUiStore,
} from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import type { MochiSettings } from "@/lib/schemas/settings";
import type { ProviderUsageState } from "@/lib/schemas/usage";
import { syncTrayUsage } from "@/lib/tauri/commands";

import {
  runColdStartProviderRefreshSequence,
  shouldRefreshEnabledProvidersOnBoot,
} from "./use-cold-start-provider-refresh";

vi.mock("@/lib/tauri/commands", () => ({
  getSettings: vi.fn<() => Promise<MochiSettings>>(() =>
    Promise.resolve({
      update_channel: "stable",
      refresh_interval_seconds: 300,
      enabled_providers: ["codex"],
      show_notifications: true,
      provider_configs: {},
    }),
  ),
  getUsageStates: vi.fn<() => Promise<ProviderUsageState[]>>(() => Promise.resolve([])),
  refreshEnabledProviders: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  syncTrayUsage: vi.fn<() => Promise<void>>(() => Promise.resolve()),
}));

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime("2026-06-20T12:00:00Z");
  vi.mocked(syncTrayUsage).mockClear();
  useTrayUiStore.getState().setSelectedTab("overview");
});

afterEach(() => {
  vi.useRealTimers();
});

function settings(enabledProviders: MochiSettings["enabled_providers"]): MochiSettings {
  return {
    update_channel: "stable",
    refresh_interval_seconds: 300,
    enabled_providers: enabledProviders,
    show_notifications: true,
    provider_configs: {},
  };
}

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
});

describe("shouldRefreshEnabledProvidersOnBoot fresh timestamps", () => {
  it("does not refresh when all enabled providers have fresh data within refresh interval", () => {
    const recent = new Date().toISOString();
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [state("codex", "fresh", recent)]),
    ).toBe(false);
  });

  it("refreshes when fresh data is older than refresh interval", () => {
    const oldDate = new Date(Date.now() - 301_000).toISOString();
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [state("codex", "fresh", oldDate)]),
    ).toBe(true);
  });

  it("does not refresh when fresh data is exactly as old as the refresh interval", () => {
    const thresholdDate = new Date(Date.now() - 300_000).toISOString();
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [
        state("codex", "fresh", thresholdDate),
      ]),
    ).toBe(false);
  });

  it("refreshes when a fresh timestamp is malformed", () => {
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [
        state("codex", "fresh", "not-a-timestamp"),
      ]),
    ).toBe(true);
  });

  it("refreshes when a fresh timestamp is in the future", () => {
    const futureDate = new Date(Date.now() + 60_000).toISOString();
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [
        state("codex", "fresh", futureDate),
      ]),
    ).toBe(true);
  });
});

describe("shouldRefreshEnabledProvidersOnBoot exclusions", () => {
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

describe("useColdStartProviderRefresh boot guard", () => {
  it("still detects fetching providers during the initial boot check", () => {
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex", "gemini"]), [state("gemini")]),
    ).toBe(true);
  });
});

describe("runColdStartProviderRefreshSequence", () => {
  it("cold start refresh syncs the selected provider from the store after invalidating cache", async () => {
    useTrayUiStore.getState().setSelectedTab("codex");

    await runColdStartProviderRefreshSequence(
      { ...settings(["codex"]), enabled_providers: ["codex"] },
      () => Promise.resolve(),
      () => Promise.resolve(),
      syncCurrentTrayUsage,
    );

    expect(syncTrayUsage).toHaveBeenCalledWith("codex");
  });
});
