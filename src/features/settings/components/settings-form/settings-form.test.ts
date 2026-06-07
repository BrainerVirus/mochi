import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import { DEFAULT_MOCHI_SETTINGS } from "@/lib/schemas/settings";

import { resolveSettingsFormState } from "@/features/settings/components/settings-form-state";

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

describe("SettingsForm source", () => {
  it("does not render the linux tray hint in the normal settings form", () => {
    const source = readFileSync(
      resolve("src/features/settings/components/settings-form/settings-form.tsx"),
      "utf8",
    );
    expect(source).not.toContain("LinuxTrayHint");
    expect(source).not.toContain("data-linux-tray-hint");
  });
});
