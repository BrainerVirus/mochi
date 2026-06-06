import { beforeEach, describe, expect, it, vi } from "vitest";

import type { MochiSettings } from "@/lib/schemas/settings";
import type { ProviderUsageState } from "@/lib/schemas/usage";
import { syncCurrentTrayUsage, useTrayUiStore } from "@/lib/stores/tray-ui-store";
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
  vi.mocked(syncTrayUsage).mockClear();
  useTrayUiStore.getState().setSelectedTab("overview");
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

describe("shouldRefreshEnabledProvidersOnBoot", () => {
  it("refreshes when an enabled provider is in fetching state", () => {
    expect(shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [state("codex")])).toBe(true);
  });

  it("does not refresh when enabled providers already have fresh state", () => {
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [state("codex", "fresh")]),
    ).toBe(false);
  });

  it("does not refresh when no providers are enabled", () => {
    expect(shouldRefreshEnabledProvidersOnBoot(settings([]), [state("codex")])).toBe(false);
  });

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
