import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { syncTrayUsage } from "@/lib/tauri/commands";

import {
  readStoredTab,
  resolveValidTraySelection,
  syncCurrentTrayUsage,
  useTrayUiStore,
} from "./tray-ui-store";

vi.mock("@/lib/tauri/commands", () => ({
  syncTrayUsage: vi.fn<() => Promise<void>>(() => Promise.resolve()),
}));

beforeEach(() => {
  vi.mocked(syncTrayUsage).mockClear();
  useTrayUiStore.getState().setSelectedTab("overview");
});

describe("readStoredTab", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("falls back to overview when window is undefined (SSR / Node test env)", () => {
    expect(readStoredTab()).toBe("overview");
  });

  it("returns the stored tab when window.__MOCHI_SELECTED_TAB__ is a valid provider", () => {
    vi.stubGlobal("window", { __MOCHI_SELECTED_TAB__: "codex" });
    expect(readStoredTab()).toBe("codex");
  });

  it("returns overview when window.__MOCHI_SELECTED_TAB__ is overview", () => {
    vi.stubGlobal("window", { __MOCHI_SELECTED_TAB__: "overview" });
    expect(readStoredTab()).toBe("overview");
  });

  it("falls back to overview when window.__MOCHI_SELECTED_TAB__ is an invalid string", () => {
    vi.stubGlobal("window", { __MOCHI_SELECTED_TAB__: "invalid-provider" });
    expect(readStoredTab()).toBe("overview");
  });
});

describe("resolveValidTraySelection", () => {
  it("keeps an enabled provider selected even when snapshots are missing", () => {
    expect(resolveValidTraySelection("codex", ["codex"])).toBe("codex");
  });

  it("falls back to overview when selected provider is disabled", () => {
    expect(resolveValidTraySelection("codex", ["cursor"])).toBe("overview");
  });

  it("keeps overview selected", () => {
    expect(resolveValidTraySelection("overview", ["codex"])).toBe("overview");
  });

  it("persists overview only when the stored provider is no longer enabled", async () => {
    expect(resolveValidTraySelection("codex", [])).toBe("overview");
    expect(resolveValidTraySelection("codex", ["codex"])).toBe("codex");
  });

  it("passes the current selected provider to the native tray sync", async () => {
    useTrayUiStore.getState().setSelectedTab("codex");

    await syncCurrentTrayUsage({ enabled_providers: ["codex"] });

    expect(syncTrayUsage).toHaveBeenCalledWith("codex");
  });

  it("passes overview only when the selected provider is disabled", async () => {
    useTrayUiStore.getState().setSelectedTab("codex");

    await syncCurrentTrayUsage({ enabled_providers: ["cursor"] });

    expect(syncTrayUsage).toHaveBeenCalledWith("overview");
    expect(useTrayUiStore.getState().selectedTab).toBe("overview");
  });
});
