import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  syncCurrentTrayUsage,
  useTrayUiStore,
} from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { queryKeys } from "@/lib/query/keys";
import { DEFAULT_MOCHI_SETTINGS, type MochiSettings } from "@/lib/schemas/settings";
import type { ProviderId, ProviderUsageState } from "@/lib/schemas/usage";
import { syncTrayUsage } from "@/lib/tauri/commands";

import { reconcileSettingsSaveSuccess } from "./use-tray-events";

vi.mock("@/lib/tauri/commands", () => ({
  getSettings: vi.fn<() => Promise<MochiSettings>>(() => Promise.resolve(DEFAULT_MOCHI_SETTINGS)),
  openAppWindow: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  saveSettings: vi.fn<(settings: MochiSettings) => Promise<MochiSettings>>((settings) =>
    Promise.resolve(settings),
  ),
  syncTrayUpdateChannel: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  syncTrayUsage: vi.fn<() => Promise<void>>(() => Promise.resolve()),
}));

const sampleUsageState: ProviderUsageState = {
  provider: "codex",
  kind: "fresh",
  snapshot: null,
  health: "ok",
  updated_at: "2026-06-13T12:00:00Z",
};

function usageState(provider: ProviderId): ProviderUsageState {
  return { ...sampleUsageState, provider };
}

function isProviderUsageState(value: unknown): value is ProviderUsageState {
  return typeof value === "object" && value !== null && "provider" in value;
}

function createUsageTrackingQueryClient(initialStates: ProviderUsageState[]) {
  let usageCache = initialStates;

  return {
    usageCache: () => usageCache,
    queryClient: {
      setQueryData: (_key: readonly unknown[], data: unknown) => {
        if (Array.isArray(data) && data.every(isProviderUsageState)) {
          usageCache = data;
        }
      },
      getQueryData: (queryKey: readonly unknown[]) =>
        queryKey === queryKeys.usageSnapshots ? usageCache : undefined,
      invalidateQueries: () => Promise.resolve(),
    },
  };
}

beforeEach(() => {
  vi.mocked(syncTrayUsage).mockClear();
  useTrayUiStore.getState().setSelectedTab("overview");
});

describe("settings save reconciliation — cache and channel updates", () => {
  it("invalidates cached usage and syncs update channel after settings save", async () => {
    const calls: string[] = [];
    const queryClient = {
      setQueryData: (queryKey: readonly unknown[]) => {
        calls.push(`set:${queryKey.join("/")}`);
      },
      getQueryData: () => undefined,
      invalidateQueries: ({ queryKey }: { queryKey: readonly unknown[] }) => {
        calls.push(`invalidate:${queryKey.join("/")}`);
        return Promise.resolve();
      },
    };

    await reconcileSettingsSaveSuccess(queryClient, DEFAULT_MOCHI_SETTINGS, (channel) => {
      calls.push(`sync-channel:${channel}`);
      return Promise.resolve();
    });

    expect(calls).toEqual([
      `set:${queryKeys.settings.join("/")}`,
      "sync-channel:stable",
      `invalidate:${queryKeys.usageSnapshots.join("/")}`,
    ]);
    expect(syncTrayUsage).not.toHaveBeenCalled();
  });

  it("invalidates usage when a provider is newly enabled", async () => {
    const calls: string[] = [];
    const queryClient = {
      setQueryData: () => undefined,
      getQueryData: () => undefined,
      invalidateQueries: ({ queryKey }: { queryKey: readonly unknown[] }) => {
        calls.push(`invalidate:${queryKey.join("/")}`);
        return Promise.resolve();
      },
    };

    await reconcileSettingsSaveSuccess(
      queryClient,
      { ...DEFAULT_MOCHI_SETTINGS, enabled_providers: ["codex", "claude"] },
      () => Promise.resolve(),
    );

    expect(calls).toEqual([`invalidate:${queryKeys.usageSnapshots.join("/")}`]);
  });
});

describe("settings save reconciliation — usage cache pruning", () => {
  it("drops disabled providers from cached usage immediately", async () => {
    const { queryClient, usageCache } = createUsageTrackingQueryClient([
      sampleUsageState,
      usageState("gemini"),
    ]);

    await reconcileSettingsSaveSuccess(
      queryClient,
      { ...DEFAULT_MOCHI_SETTINGS, enabled_providers: ["codex"] },
      () => Promise.resolve(),
    );

    expect(usageCache().map((state) => state.provider)).toEqual(["codex"]);
  });

  it("clears cached usage when every provider is disabled", async () => {
    const { queryClient, usageCache } = createUsageTrackingQueryClient([
      sampleUsageState,
      usageState("gemini"),
    ]);

    await reconcileSettingsSaveSuccess(
      queryClient,
      { ...DEFAULT_MOCHI_SETTINGS, enabled_providers: [] },
      () => Promise.resolve(),
    );

    expect(usageCache()).toEqual([]);
  });

  it("uses the latest enabled list when reconciliation runs in sequence", async () => {
    const { queryClient, usageCache } = createUsageTrackingQueryClient([
      sampleUsageState,
      usageState("gemini"),
      usageState("cursor"),
    ]);

    await reconcileSettingsSaveSuccess(
      queryClient,
      { ...DEFAULT_MOCHI_SETTINGS, enabled_providers: ["codex", "gemini"] },
      () => Promise.resolve(),
    );
    expect(usageCache().map((state) => state.provider)).toEqual(["codex", "gemini"]);

    await reconcileSettingsSaveSuccess(
      queryClient,
      { ...DEFAULT_MOCHI_SETTINGS, enabled_providers: ["codex"] },
      () => Promise.resolve(),
    );
    expect(usageCache().map((state) => state.provider)).toEqual(["codex"]);
  });
});

describe("settings save reconciliation — tray tab sync", () => {
  it("resets selected tab when syncCurrentTrayUsage runs after enabled providers shrink", async () => {
    useTrayUiStore.getState().setSelectedTab("gemini");
    const queryClient = {
      setQueryData: () => undefined,
      getQueryData: () => undefined,
      invalidateQueries: () => Promise.resolve(),
    };

    await reconcileSettingsSaveSuccess(
      queryClient,
      { ...DEFAULT_MOCHI_SETTINGS, enabled_providers: ["codex"] },
      () => Promise.resolve(),
    );
    await syncCurrentTrayUsage({ enabled_providers: ["codex"] });

    expect(useTrayUiStore.getState().selectedTab).toBe("overview");
    expect(syncTrayUsage).toHaveBeenCalledWith("overview");
  });
});
