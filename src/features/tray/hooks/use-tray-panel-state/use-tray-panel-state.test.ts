// @vitest-environment happy-dom
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook } from "@testing-library/react";
import { createElement, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  useTrayUiStore,
  type TraySelectedTab,
} from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { queryKeys } from "@/lib/query/keys";
import { DEFAULT_MOCHI_SETTINGS, type MochiSettings } from "@/lib/schemas/settings";
import { saveSettings, syncTrayUsage } from "@/lib/tauri/commands";

import { persistTabChangeSettings, useTrayPanelState } from "./use-tray-panel-state";

vi.mock("@/lib/tauri/commands", () => ({
  getSettings: vi.fn<() => Promise<MochiSettings>>(() => Promise.resolve(DEFAULT_MOCHI_SETTINGS)),
  getUsageStates: vi.fn<() => Promise<unknown>>(() => Promise.resolve([])),
  openAppWindow: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  refreshAllProviders: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  refreshSingleProvider: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  saveSettings: vi.fn<(settings: MochiSettings) => Promise<MochiSettings>>((settings) =>
    Promise.resolve(settings),
  ),
  syncTrayUpdateChannel: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  syncTrayUsage: vi.fn<() => Promise<void>>(() => Promise.resolve()),
}));

vi.mock("@/features/tray/hooks/use-tray-events", () => ({
  useSettings: () => ({ data: { ...DEFAULT_MOCHI_SETTINGS } }),
}));

vi.mock("@/features/tray/hooks/use-tray-panel-refresh", () => ({
  useTrayPanelRefresh: () => ({ refreshAll: vi.fn<() => Promise<void>>(), isRefreshingAll: false }),
}));

vi.mock("@/features/usage/hooks/use-usage-data/use-usage-data", () => ({
  useUsageData: () => ({
    data: [],
    error: null,
    isError: false,
    isPending: false,
    isSuccess: true,
    isFetching: false,
  }),
}));

beforeEach(() => {
  vi.clearAllMocks();
  useTrayUiStore.getState().setSelectedTab("overview");
});

describe("persistTabChangeSettings", () => {
  const queryClient = {
    setQueryData: vi.fn<(queryKey: readonly unknown[], data: unknown) => unknown>(),
  };

  const baseSettings = { ...DEFAULT_MOCHI_SETTINGS };

  it("calls saveSettings with updated settings and updates cache on success", async () => {
    const nextTab = "codex" as const;

    await persistTabChangeSettings(queryClient, baseSettings, nextTab);

    const expected = { ...baseSettings, selected_tab: nextTab };
    expect(saveSettings).toHaveBeenCalledWith(expected);
    expect(queryClient.setQueryData).toHaveBeenCalledWith(queryKeys.settings, expected);
  });

  it("restores original settings in cache on saveSettings rejection", async () => {
    vi.mocked(saveSettings).mockRejectedValueOnce(new Error("save failed"));

    await persistTabChangeSettings(queryClient, baseSettings, "codex");

    expect(queryClient.setQueryData).toHaveBeenCalledWith(queryKeys.settings, baseSettings);
  });

  it("skips cache update when pendingTabRef has advanced (race survivor wins)", async () => {
    const ref = { current: null as TraySelectedTab | null };
    const promise1 = persistTabChangeSettings(queryClient, baseSettings, "codex", ref);
    ref.current = "cursor";
    const promise2 = persistTabChangeSettings(queryClient, baseSettings, "cursor", ref);
    await promise1;
    await promise2;

    expect(queryClient.setQueryData).toHaveBeenCalledTimes(1);
    expect(queryClient.setQueryData).toHaveBeenCalledWith(queryKeys.settings, {
      ...baseSettings,
      selected_tab: "cursor",
    });
  });

  it("skips rollback when pendingTabRef has advanced past the failing save", async () => {
    const ref = { current: null as TraySelectedTab | null };
    vi.mocked(saveSettings).mockRejectedValueOnce(new Error("save failed"));

    const promise = persistTabChangeSettings(queryClient, baseSettings, "codex", ref);
    ref.current = "cursor";
    await promise;

    expect(queryClient.setQueryData).not.toHaveBeenCalled();
  });
});

function makeWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return ({ children }: { children: ReactNode }) =>
    createElement(QueryClientProvider, { client: queryClient }, children);
}

describe("useTrayPanelState", () => {
  it("returns a referentially stable handleTabChange across renders with the same settings", () => {
    const { result, rerender } = renderHook(() => useTrayPanelState(), {
      wrapper: makeWrapper(),
    });
    const first = result.current.handleTabChange;
    rerender();
    const second = result.current.handleTabChange;
    expect(second).toBe(first);
  });

  it("does not call syncTrayUsage directly from handleTabChange", () => {
    const syncTrayUsageMock = vi.mocked(syncTrayUsage);
    syncTrayUsageMock.mockClear();
    const { result } = renderHook(() => useTrayPanelState(), {
      wrapper: makeWrapper(),
    });
    result.current.handleTabChange("codex");
    expect(syncTrayUsageMock).not.toHaveBeenCalled();
  });
});
