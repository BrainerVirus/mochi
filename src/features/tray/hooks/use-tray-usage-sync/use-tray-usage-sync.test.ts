// @vitest-environment happy-dom
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { createElement, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  syncCurrentTrayUsage,
  useTrayUiStore,
} from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { DEFAULT_MOCHI_SETTINGS } from "@/lib/schemas/settings";
import { syncTrayUsage } from "@/lib/tauri/commands";

import { useTrayUsageSync } from "./use-tray-usage-sync";

vi.mock("@/lib/tauri/commands", () => ({
  syncTrayUsage: vi.fn<() => Promise<void>>(() => Promise.resolve()),
}));

vi.mock("@/features/tray/hooks/use-tray-events", () => ({
  useSettings: () => ({ data: DEFAULT_MOCHI_SETTINGS }),
}));

vi.mock("@/features/usage/hooks/use-usage-data/use-usage-data", () => ({
  useUsageData: () => ({
    data: [],
    isSuccess: true,
  }),
}));

vi.mock("@/features/tray/lib/stores/tray-ui-store/tray-ui-store", async (importOriginal) => {
  const original =
    await importOriginal<typeof import("@/features/tray/lib/stores/tray-ui-store/tray-ui-store")>();
  return {
    ...original,
    syncCurrentTrayUsage: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  };
});

beforeEach(() => {
  vi.clearAllMocks();
  useTrayUiStore.getState().setSelectedTab("overview");
});

function makeWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return ({ children }: { children: ReactNode }) =>
    createElement(QueryClientProvider, { client: queryClient }, children);
}

describe("useTrayUsageSync", () => {
  it("syncs once per tab change, not once per mounted window effect", () => {
    const { rerender } = renderHook(() => useTrayUsageSync(), {
      wrapper: makeWrapper(),
    });

    const syncMock = vi.mocked(syncTrayUsage);
    syncMock.mockClear();
    // Mounting the same hook twice (widget + panel render trees) must not
    // multiply the initial sync.
    rerender();

    act(() => {
      useTrayUiStore.getState().setSelectedTab("codex");
    });

    expect(syncTrayUsage).toHaveBeenCalledTimes(1);
    expect(syncTrayUsage).toHaveBeenCalledWith("codex");
  });

  it("refresh-complete path still syncs the reconciled selection", () => {
    renderHook(() => useTrayUsageSync(), { wrapper: makeWrapper() });
    expect(syncCurrentTrayUsage).toHaveBeenCalledWith(DEFAULT_MOCHI_SETTINGS);
  });
});
