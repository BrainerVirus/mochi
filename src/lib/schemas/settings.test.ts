import { describe, expect, it } from "vitest";

import { MochiSettingsSchema } from "./settings";

describe("MochiSettingsSchema", () => {
  it("accepts stable defaults from the backend", () => {
    const parsed = MochiSettingsSchema.parse({
      update_channel: "stable",
      refresh_interval_seconds: 300,
      enabled_providers: ["codex", "claude"],
      show_notifications: true,
    });

    expect(parsed.update_channel).toBe("stable");
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
