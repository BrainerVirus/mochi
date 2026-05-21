import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn<(command: string, args?: Record<string, unknown>) => Promise<unknown>>(),
}));

import { invoke } from "@tauri-apps/api/core";

import { DEFAULT_MOCHI_SETTINGS } from "@/lib/schemas/settings";

import { getSettings } from "./commands";

describe("getSettings", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("returns defaults without calling Tauri when the runtime is unavailable", async () => {
    await expect(getSettings()).resolves.toEqual(DEFAULT_MOCHI_SETTINGS);
    expect(invoke).not.toHaveBeenCalled();
  });
});
