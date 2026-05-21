import { describe, expect, it } from "vitest";

import { DEFAULT_MOCHI_SETTINGS } from "@/lib/schemas/settings";

import { resolveSettingsFormState } from "./settings-form-state";

describe("resolveSettingsFormState", () => {
  it("keeps the editor shell while settings are still loading", () => {
    expect(
      resolveSettingsFormState({
        data: undefined,
        isPending: true,
        isError: false,
      }),
    ).toEqual({
      kind: "editor",
      settings: DEFAULT_MOCHI_SETTINGS,
      isLoading: true,
    });
  });

  it("uses loaded settings once the query resolves", () => {
    const loaded = {
      ...DEFAULT_MOCHI_SETTINGS,
      refresh_interval_seconds: 120,
    };

    expect(
      resolveSettingsFormState({
        data: loaded,
        isPending: false,
        isError: false,
      }),
    ).toEqual({
      kind: "editor",
      settings: loaded,
      isLoading: false,
    });
  });

  it("surfaces query errors without swapping to the tabs shell", () => {
    expect(
      resolveSettingsFormState({
        data: undefined,
        isPending: false,
        isError: true,
      }),
    ).toEqual({ kind: "error" });
  });
});
