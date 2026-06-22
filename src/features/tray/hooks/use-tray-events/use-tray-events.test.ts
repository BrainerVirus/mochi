// @vitest-environment happy-dom
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import { createElement, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useTrayUiStore } from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { queryKeys } from "@/lib/query/keys";
import { DEFAULT_MOCHI_SETTINGS, type MochiSettings } from "@/lib/schemas/settings";
import type { ProviderUsageState } from "@/lib/schemas/usage";
import { saveSettings, syncTrayUpdateChannel, syncTrayUsage } from "@/lib/tauri/commands";

import {
  cacheSavedSettings,
  handleSetTabEvent,
  handleSettingsSaveSuccess,
  handleTraySetChannelEvent,
  handleUsageRefreshComplete,
  useSaveSettings,
  useTrayEvents,
} from "./use-tray-events";

const listenHandlers = vi.hoisted(() => new Map<string, (event: { payload: unknown }) => void>());

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn<
    (eventName: string, handler: (event: { payload: unknown }) => void) => Promise<() => void>
  >((eventName, handler) => {
    listenHandlers.set(eventName, handler);
    return Promise.resolve(() => {
      listenHandlers.delete(eventName);
    });
  }),
}));

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn<() => void>(),
}));

vi.mock("@/lib/tauri/window-events", () => ({
  shouldHandleAppNavigateEvent: () => false,
  shouldHandleTrayNavigateEvent: () => false,
}));

vi.mock("@/lib/tauri/diagnostics", () => ({
  reportFrontendError: vi.fn<() => Promise<void>>(() => Promise.resolve()),
}));

vi.mock("@/lib/tauri/commands", () => ({
  getSettings: vi.fn<() => Promise<MochiSettings>>(() => Promise.resolve(DEFAULT_MOCHI_SETTINGS)),
  openAppWindow: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  saveSettings: vi.fn<(settings: MochiSettings) => Promise<MochiSettings>>((settings) =>
    Promise.resolve(settings),
  ),
  syncTrayUpdateChannel: vi.fn<(channel: MochiSettings["update_channel"]) => Promise<void>>(() =>
    Promise.resolve(),
  ),
  syncTrayUsage: vi.fn<() => Promise<void>>(() => Promise.resolve()),
}));

const sampleUsageState: ProviderUsageState = {
  provider: "codex",
  kind: "fresh",
  snapshot: null,
  health: "ok",
  updated_at: "2026-06-13T12:00:00Z",
};

function createWrapper(queryClient: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(QueryClientProvider, { client: queryClient }, children);
  };
}

beforeEach(() => {
  listenHandlers.clear();
  vi.mocked(saveSettings).mockClear();
  vi.mocked(syncTrayUpdateChannel).mockClear();
  vi.mocked(syncTrayUsage).mockClear();
  useTrayUiStore.getState().setSelectedTab("overview");
});

describe("useTrayEvents settings-changed listener", () => {
  it("registers a listener that reconciles valid payloads", async () => {
    const queryClient = new QueryClient();

    renderHook(() => useTrayEvents(), {
      wrapper: createWrapper(queryClient),
    });

    await waitFor(() => {
      expect(listenHandlers.has("settings-changed")).toBe(true);
    });

    listenHandlers.get("settings-changed")?.({ payload: DEFAULT_MOCHI_SETTINGS });

    await waitFor(() => {
      expect(queryClient.getQueryData(queryKeys.settings)).toEqual(DEFAULT_MOCHI_SETTINGS);
    });
  });
});

describe("useSaveSettings", () => {
  it("reconciles immediately in the saving webview", async () => {
    const queryClient = new QueryClient();
    const { result } = renderHook(() => useSaveSettings(), {
      wrapper: createWrapper(queryClient),
    });

    await result.current.mutateAsync({
      ...DEFAULT_MOCHI_SETTINGS,
      enabled_providers: ["codex"],
    });

    expect(queryClient.getQueryData(queryKeys.settings)).toEqual({
      ...DEFAULT_MOCHI_SETTINGS,
      enabled_providers: ["codex"],
    });
    expect(syncTrayUpdateChannel).toHaveBeenCalledWith("stable");
  });
});

describe("handleSettingsSaveSuccess", () => {
  it("writes settings cache before running reconcile", async () => {
    const calls: string[] = [];
    const queryClient = {
      setQueryData: (queryKey: readonly unknown[]) => {
        calls.push(`set:${queryKey.join("/")}`);
      },
      getQueryData: () => undefined,
      invalidateQueries: () => Promise.resolve(),
    };

    handleSettingsSaveSuccess(queryClient, DEFAULT_MOCHI_SETTINGS, async () => {
      calls.push("reconcile");
    });

    expect(calls[0]).toBe(`set:${queryKeys.settings.join("/")}`);
    await waitFor(() => {
      expect(calls).toEqual([`set:${queryKeys.settings.join("/")}`, "reconcile"]);
    });
  });
});

describe("cacheSavedSettings", () => {
  it("writes settings into the query cache", () => {
    const queryClient = new QueryClient();
    cacheSavedSettings(queryClient, DEFAULT_MOCHI_SETTINGS);
    expect(queryClient.getQueryData(queryKeys.settings)).toEqual(DEFAULT_MOCHI_SETTINGS);
  });
});

describe("handleTraySetChannelEvent", () => {
  it("persists channel changes and reconciles locally as emit fallback", async () => {
    const queryClient = new QueryClient();
    queryClient.setQueryData(queryKeys.settings, DEFAULT_MOCHI_SETTINGS);

    handleTraySetChannelEvent("unstable", queryClient);

    await waitFor(() => {
      expect(saveSettings).toHaveBeenCalledWith({
        ...DEFAULT_MOCHI_SETTINGS,
        update_channel: "unstable",
      });
      expect(syncTrayUpdateChannel).toHaveBeenCalledWith("unstable");
    });
  });

  it("does nothing when settings are not cached", () => {
    const queryClient = new QueryClient();

    handleTraySetChannelEvent("unstable", queryClient);

    expect(saveSettings).not.toHaveBeenCalled();
  });
});

describe("handleSetTabEvent", () => {
  it("updates the store tab from a provider id payload", () => {
    useTrayUiStore.getState().setSelectedTab("overview");
    handleSetTabEvent("codex");
    expect(useTrayUiStore.getState().selectedTab).toBe("codex");
  });

  it("updates the store tab from an overview payload", () => {
    useTrayUiStore.getState().setSelectedTab("codex");
    handleSetTabEvent("overview");
    expect(useTrayUiStore.getState().selectedTab).toBe("overview");
  });

  it("falls back to overview on invalid payload", () => {
    useTrayUiStore.getState().setSelectedTab("codex");
    handleSetTabEvent("nonexistent-provider");
    expect(useTrayUiStore.getState().selectedTab).toBe("overview");
  });
});

describe("usage refresh complete handler", () => {
  it("sets query data and does not sync tray when no settings cached", () => {
    const calls: string[] = [];

    handleUsageRefreshComplete(
      [sampleUsageState],
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

  it("sets query data and syncs tray usage when settings are cached", () => {
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
