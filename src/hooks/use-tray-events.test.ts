import { describe, expect, it } from "vitest";

import { queryKeys } from "@/lib/query/keys";
import { DEFAULT_MOCHI_SETTINGS } from "@/lib/schemas/settings";

import {
  reconcileSettingsSaveSuccess,
  shouldRunProviderRefreshForTrayEvent,
} from "./use-tray-events";

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

    await reconcileSettingsSaveSuccess(queryClient, DEFAULT_MOCHI_SETTINGS, () => {
      calls.push("sync");
      return Promise.resolve();
    });

    expect(calls).toEqual([
      `set:${queryKeys.settings.join("/")}`,
      `invalidate:${queryKeys.usageSnapshots.join("/")}`,
      "sync",
    ]);
  });
});
