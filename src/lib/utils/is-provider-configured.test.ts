import { describe, expect, it } from "vitest";

import type { UsageSnapshot } from "@/lib/schemas/usage";

import {
  STATIC_SNAPSHOT_EPOCH,
  filterConfiguredSnapshots,
  isProviderConfigured,
} from "./is-provider-configured";

function snapshot(overrides: Partial<UsageSnapshot> = {}): UsageSnapshot {
  return {
    provider: "codex",
    primary: {
      label: "Session",
      used_percent: 25,
      remaining_percent: 75,
      resets_at: "2026-05-21T18:00:00Z",
    },
    secondary: null,
    updated_at: "2026-05-21T12:00:00Z",
    source: "codex-cli",
    ...overrides,
  };
}

describe("isProviderConfigured", () => {
  it("returns false for static placeholder snapshots", () => {
    expect(
      isProviderConfigured(
        snapshot({
          updated_at: STATIC_SNAPSHOT_EPOCH,
          source: "Claude",
          primary: {
            label: "Session",
            used_percent: 0,
            remaining_percent: 100,
            resets_at: null,
          },
        }),
      ),
    ).toBe(false);
  });

  it("returns true when updated_at reflects a real fetch", () => {
    expect(isProviderConfigured(snapshot())).toBe(true);
  });
});

describe("filterConfiguredSnapshots", () => {
  it("removes unconfigured static placeholders", () => {
    const configured = snapshot({ provider: "codex" });
    const unconfigured = snapshot({
      provider: "claude",
      updated_at: STATIC_SNAPSHOT_EPOCH,
      source: "Claude",
    });

    expect(filterConfiguredSnapshots([configured, unconfigured])).toEqual([configured]);
  });
});
