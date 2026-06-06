import { beforeEach, describe, expect, it, vi } from "vitest";

import { queryKeys } from "@/lib/query/keys";
import { DEFAULT_MOCHI_SETTINGS, type MochiSettings } from "@/lib/schemas/settings";
import { syncCurrentTrayUsage, useTrayUiStore } from "@/lib/stores/tray-ui-store";
import { syncTrayUsage } from "@/lib/tauri/commands";

import {
  reconcileSettingsSaveSuccess,
  runTrayRefreshEventSequence,
  shouldRunProviderRefreshForTrayEvent,
} from "./use-tray-events";

vi.mock("@/lib/tauri/commands", () => ({
  getSettings: vi.fn<() => Promise<MochiSettings>>(() => Promise.resolve(DEFAULT_MOCHI_SETTINGS)),
  openAppWindow: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  refreshEnabledProviders: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  saveSettings: vi.fn<(settings: MochiSettings) => Promise<MochiSettings>>((settings) =>
    Promise.resolve(settings),
  ),
  syncTrayUpdateChannel: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  syncTrayUsage: vi.fn<() => Promise<void>>(() => Promise.resolve()),
}));

beforeEach(() => {
  vi.mocked(syncTrayUsage).mockClear();
  useTrayUiStore.getState().setSelectedTab("overview");
});

describe("tray event refresh policy", () => {
  it("runs a real provider refresh before resyncing tray usage", () => {
    expect(shouldRunProviderRefreshForTrayEvent("tray-refresh")).toBe(true);
  });

  it("invalidates cached usage and syncs tray usage after settings save", async () => {
    const calls: string[] = [];
    const queryClient = {
      setQueryData: (queryKey: readonly unknown[]) => {
        calls.push(`set:${queryKey.join("/")}`);
      },
      invalidateQueries: ({ queryKey }: { queryKey: readonly unknown[] }) => {
        calls.push(`invalidate:${queryKey.join("/")}`);
        return Promise.resolve();
      },
    };

    await reconcileSettingsSaveSuccess(
      queryClient,
      DEFAULT_MOCHI_SETTINGS,
      () => {
        calls.push("sync-usage");
        return Promise.resolve();
      },
      (channel) => {
        calls.push(`sync-channel:${channel}`);
        return Promise.resolve();
      },
    );

    expect(calls).toEqual([
      `set:${queryKeys.settings.join("/")}`,
      "sync-channel:stable",
      `invalidate:${queryKeys.usageSnapshots.join("/")}`,
      "sync-usage",
    ]);
  });

  it("native tray refresh syncs the selected provider from the store", async () => {
    useTrayUiStore.getState().setSelectedTab("codex");
    const queryClient = {
      invalidateQueries: () => Promise.resolve(),
    };

    await runTrayRefreshEventSequence(
      queryClient,
      { ...DEFAULT_MOCHI_SETTINGS, enabled_providers: ["codex"] },
      () => Promise.resolve(),
      syncCurrentTrayUsage,
    );

    expect(syncTrayUsage).toHaveBeenCalledWith("codex");
  });

  it("settings save syncs the selected provider from the store", async () => {
    useTrayUiStore.getState().setSelectedTab("codex");
    const queryClient = {
      setQueryData: () => undefined,
      invalidateQueries: () => Promise.resolve(),
    };

    await reconcileSettingsSaveSuccess(
      queryClient,
      { ...DEFAULT_MOCHI_SETTINGS, enabled_providers: ["codex"] },
      () => syncCurrentTrayUsage({ enabled_providers: ["codex"] }),
      () => Promise.resolve(),
    );

    expect(syncTrayUsage).toHaveBeenCalledWith("codex");
  });
});
