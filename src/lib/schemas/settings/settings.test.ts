import { describe, expect, it } from "vitest";

import { DEFAULT_MOCHI_SETTINGS, MochiSettingsSchema, PROVIDER_LABELS } from "./settings";

describe("MochiSettingsSchema", () => {
  it("ships defaults aligned with the Rust backend", () => {
    expect(DEFAULT_MOCHI_SETTINGS.update_channel).toBe("stable");
    expect(DEFAULT_MOCHI_SETTINGS.refresh_interval_seconds).toBe(300);
    expect(DEFAULT_MOCHI_SETTINGS.show_notifications).toBe(true);
    expect(DEFAULT_MOCHI_SETTINGS.enabled_providers).toEqual([]);
  });

  it("accepts stable defaults from the backend", () => {
    const parsed = MochiSettingsSchema.parse({
      update_channel: "stable",
      refresh_interval_seconds: 300,
      enabled_providers: ["codex", "claude"],
      show_notifications: true,
    });

    expect(parsed.update_channel).toBe("stable");
  });

  it("accepts commandcode as an enabled provider", () => {
    const parsed = MochiSettingsSchema.parse({
      update_channel: "stable",
      refresh_interval_seconds: 300,
      enabled_providers: ["commandcode"],
      show_notifications: true,
    });

    expect(parsed.enabled_providers).toEqual(["commandcode"]);
    expect(PROVIDER_LABELS.commandcode).toBe("Command Code");
  });

  it("rejects refresh intervals below thirty seconds", () => {
    const result = MochiSettingsSchema.safeParse({
      update_channel: "stable",
      refresh_interval_seconds: 10,
      enabled_providers: ["codex"],
      show_notifications: true,
    });

    expect(result.success).toBe(false);
  });
});
