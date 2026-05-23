import { describe, expect, it } from "vitest";

import type { UsageSnapshot } from "@/lib/schemas/usage";

import { STATIC_SNAPSHOT_EPOCH } from "./is-provider-configured";
import { buildTrayPanelTabs, filterSnapshotsForTrayPanel } from "./tray-panel-tabs";

function snapshot(
  provider: UsageSnapshot["provider"],
  usedPercent: number,
  configured = true,
): UsageSnapshot {
  return {
    provider,
    primary: {
      label: "Session",
      used_percent: usedPercent,
      remaining_percent: 100 - usedPercent,
      resets_at: null,
    },
    secondary: null,
    extra_windows: [],
    updated_at: configured ? "2026-05-21T12:00:00Z" : STATIC_SNAPSHOT_EPOCH,
    source: configured ? "codex-cli" : "Claude",
    health: "ok",
    is_stale: false,
  };
}

describe("buildTrayPanelTabs", () => {
  it("includes overview plus one tab per configured provider snapshot", () => {
    const tabs = buildTrayPanelTabs([snapshot("codex", 40), snapshot("cursor", 72, false)]);

    expect(tabs.map((tab) => tab.id)).toEqual(["overview", "codex"]);
    expect(tabs[0]?.label).toBe("Overview");
    expect(tabs[1]?.label).toBe("Codex");
  });

  it("only includes enabled and configured provider tabs", () => {
    const tabs = buildTrayPanelTabs(
      [snapshot("codex", 40), snapshot("cursor", 72, false), snapshot("claude", 10)],
      ["codex", "claude", "cursor"],
    );

    expect(tabs.map((tab) => tab.id)).toEqual(["overview", "codex", "claude"]);
  });
});

describe("filterSnapshotsForTrayPanel", () => {
  it("returns only enabled configured snapshots", () => {
    const filtered = filterSnapshotsForTrayPanel(
      [snapshot("codex", 40), snapshot("cursor", 72, false), snapshot("claude", 10)],
      ["codex", "claude", "cursor"],
    );

    expect(filtered.map((entry) => entry.provider)).toEqual(["codex", "claude"]);
  });
});
