import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn<(command: string, args?: Record<string, unknown>) => Promise<unknown>>(),
}));

import { invoke } from "@tauri-apps/api/core";

import { DEFAULT_MOCHI_SETTINGS } from "@/lib/schemas/settings";

import { getSettings, refreshEnabledProviders } from "./commands";

describe("getSettings", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("returns defaults without calling Tauri when the runtime is unavailable", async () => {
    await expect(getSettings()).resolves.toEqual(DEFAULT_MOCHI_SETTINGS);
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("refreshEnabledProviders", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("invokes the bulk refresh command and validates usage snapshots", async () => {
    vi.mocked(invoke).mockResolvedValue([
      {
        provider: "codex",
        primary: {
          label: "Session",
          used_percent: 25,
          remaining_percent: 75,
          resets_at: null,
        },
        secondary: null,
        updated_at: "2026-05-31T12:00:00Z",
        source: "test",
        health: "ok",
        is_stale: false,
      },
    ]);

    const snapshots = await refreshEnabledProviders();

    expect(invoke).toHaveBeenCalledWith("refresh_enabled_providers");
    expect(snapshots[0]?.provider).toBe("codex");
  });
});
