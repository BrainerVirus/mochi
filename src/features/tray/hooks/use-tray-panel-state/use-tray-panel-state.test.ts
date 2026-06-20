import { beforeEach, describe, expect, it, vi } from "vitest";

import type { TraySelectedTab } from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { queryKeys } from "@/lib/query/keys";
import { DEFAULT_MOCHI_SETTINGS, type MochiSettings } from "@/lib/schemas/settings";
import { saveSelectedTab } from "@/lib/tauri/commands";

import { persistTabChangeSettings } from "./use-tray-panel-state";

vi.mock("@/lib/tauri/commands", () => ({
  getSettings: vi.fn<() => Promise<MochiSettings>>(() => Promise.resolve(DEFAULT_MOCHI_SETTINGS)),
  getUsageStates: vi.fn<() => Promise<unknown>>(() => Promise.resolve([])),
  openAppWindow: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  refreshAllProviders: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  refreshSingleProvider: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  saveSelectedTab: vi.fn<(selectedTab: TraySelectedTab) => Promise<MochiSettings>>((selectedTab) =>
    Promise.resolve({ ...DEFAULT_MOCHI_SETTINGS, selected_tab: selectedTab }),
  ),
  syncTrayUpdateChannel: vi.fn<() => Promise<void>>(() => Promise.resolve()),
}));

beforeEach(() => {
  vi.clearAllMocks();
});

describe("persistTabChangeSettings", () => {
  const queryClient = {
    setQueryData: vi.fn<(queryKey: readonly unknown[], data: unknown) => unknown>(),
  };

  const baseSettings = { ...DEFAULT_MOCHI_SETTINGS };

  it("saves only the selected tab and caches the returned settings", async () => {
    const nextTab = "codex" as const;

    await persistTabChangeSettings(queryClient, baseSettings, nextTab);

    const expected = { ...baseSettings, selected_tab: nextTab };
    expect(saveSelectedTab).toHaveBeenCalledWith(nextTab);
    expect(queryClient.setQueryData).toHaveBeenCalledWith(queryKeys.settings, expected);
  });

  it("restores original settings in cache on saveSelectedTab rejection", async () => {
    vi.mocked(saveSelectedTab).mockRejectedValueOnce(new Error("save failed"));

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
    vi.mocked(saveSelectedTab).mockRejectedValueOnce(new Error("save failed"));

    const promise = persistTabChangeSettings(queryClient, baseSettings, "codex", ref);
    ref.current = "cursor";
    await promise;

    expect(queryClient.setQueryData).not.toHaveBeenCalled();
  });
});
