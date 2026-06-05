import { describe, expect, it } from "vitest";

import type { MochiSettings } from "@/lib/schemas/settings";
import type { ProviderUsageState } from "@/lib/schemas/usage";

import { shouldRefreshEnabledProvidersOnBoot } from "./use-cold-start-provider-refresh";

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
});
