import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  syncCurrentTrayUsage,
  useTrayUiStore as trayUiStore,
} from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { queryKeys } from "@/lib/query/keys";
import { DEFAULT_MOCHI_SETTINGS, type MochiSettings } from "@/lib/schemas/settings";
import { syncTrayUsage } from "@/lib/tauri/commands";

import {
  handleSetTabEvent,
  handleUsageRefreshComplete,
  reconcileSettingsSaveSuccess,
} from "./use-tray-events";

vi.mock("@/lib/tauri/commands", () => ({
  getSettings: vi.fn<() => Promise<MochiSettings>>(() => Promise.resolve(DEFAULT_MOCHI_SETTINGS)),
  openAppWindow: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  saveSettings: vi.fn<(settings: MochiSettings) => Promise<MochiSettings>>((settings) =>
    Promise.resolve(settings),
  ),
  syncTrayUpdateChannel: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  syncTrayUsage: vi.fn<() => Promise<void>>(() => Promise.resolve()),
}));

beforeEach(() => {
  vi.mocked(syncTrayUsage).mockClear();
  trayUiStore.getState().setSelectedTab("overview");
});

describe("settings save reconciliation", () => {
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

  it("syncs the selected provider from the store", async () => {
    trayUiStore.getState().setSelectedTab("codex");
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

describe("handleSetTabEvent", () => {
  it("updates the store tab from a provider id payload", () => {
    trayUiStore.getState().setSelectedTab("overview");
    handleSetTabEvent("codex");
    expect(trayUiStore.getState().selectedTab).toBe("codex");
  });

  it("updates the store tab from an overview payload", () => {
    trayUiStore.getState().setSelectedTab("codex");
    handleSetTabEvent("overview");
    expect(trayUiStore.getState().selectedTab).toBe("overview");
  });

  it("falls back to overview on invalid payload", () => {
    trayUiStore.getState().setSelectedTab("codex");
    handleSetTabEvent("nonexistent-provider");
    expect(trayUiStore.getState().selectedTab).toBe("overview");
  });
});

describe("usage refresh complete handler", () => {
  it("sets query data and does not sync tray when no settings cached", async () => {
    const calls: string[] = [];

    handleUsageRefreshComplete(
      [
        {
          provider: "codex" as const,
          kind: "fresh" as const,
          snapshot: null,
          health: "ok" as const,
          updated_at: "2026-06-13T12:00:00Z",
        },
      ],
      (_key, _data) => {
        calls.push("set-data");
      },
      () => {
        calls.push("get-settings");
        return undefined;
      },
    );

    expect(calls).toContain("set-data");
    expect(calls).toContain("get-settings");
    expect(syncTrayUsage).not.toHaveBeenCalled();
  });

  it("sets query data and syncs tray usage when settings are cached", async () => {
    const calls: string[] = [];

    handleUsageRefreshComplete(
      [],
      (_key, _data) => {
        calls.push("set-data");
      },
      () => {
        calls.push("get-settings");
        return { enabled_providers: ["claude"] };
      },
    );

    expect(calls).toContain("set-data");
    expect(calls).toContain("get-settings");
    expect(syncTrayUsage).toHaveBeenCalledWith("overview");
  });
});
