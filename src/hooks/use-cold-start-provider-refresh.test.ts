import { describe, expect, it } from "vitest";

import type { MochiSettings } from "@/lib/schemas/settings";
import type { UsageSnapshot } from "@/lib/schemas/usage";

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

function snapshot(provider: UsageSnapshot["provider"], source = "credentials-detected") {
  return {
    provider,
    primary: {
      label: "Pending fetch",
      used_percent: 100,
      remaining_percent: 0,
      resets_at: null,
    },
    secondary: null,
    updated_at: "2026-05-31T12:00:00Z",
    source,
    health: "error",
    is_stale: false,
    error: source === "credentials-detected" ? "Usage fetch pending" : null,
    extra_windows: [],
  } satisfies UsageSnapshot;
}

describe("shouldRefreshEnabledProvidersOnBoot", () => {
  it("refreshes when an enabled provider has only a credential-detected pending snapshot", () => {
    expect(shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [snapshot("codex")])).toBe(
      true,
    );
  });

  it("does not refresh when enabled providers already have real snapshots", () => {
    expect(
      shouldRefreshEnabledProvidersOnBoot(settings(["codex"]), [snapshot("codex", "test")]),
    ).toBe(false);
  });

  it("does not refresh when no providers are enabled", () => {
    expect(shouldRefreshEnabledProvidersOnBoot(settings([]), [snapshot("codex")])).toBe(false);
  });
});
