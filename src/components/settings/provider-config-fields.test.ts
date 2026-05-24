import { describe, expect, it } from "vitest";

import type { ProviderCatalogEntry } from "@/lib/schemas/provider-catalog";

import { shouldShowProviderField } from "./provider-config-fields";

type SettingsField = ProviderCatalogEntry["settingsFields"][number];

describe("shouldShowProviderField", () => {
  const manualCookie: SettingsField = {
    key: "manual_cookie",
    label: "Manual cookie",
    kind: "manual-cookie",
  };
  const historyWindow: SettingsField = {
    key: "history_window_days",
    label: "Session cost history window",
    kind: "history-window",
  };
  const cookieSource: SettingsField = {
    key: "cookie_source",
    label: "Cookie source",
    kind: "cookie-source",
  };
  const apiKey: SettingsField = { key: "api_key", label: "API key", kind: "api-key" };

  it("hides manual cookie and history window when cookie source is auto", () => {
    const config = { cookie_source: "auto" as const };

    expect(shouldShowProviderField(manualCookie, config)).toBe(false);
    expect(shouldShowProviderField(historyWindow, config)).toBe(false);
    expect(shouldShowProviderField(cookieSource, config)).toBe(true);
  });

  it("shows manual fields when cookie source is manual", () => {
    const config = { cookie_source: "manual" as const };

    expect(shouldShowProviderField(manualCookie, config)).toBe(true);
    expect(shouldShowProviderField(historyWindow, config)).toBe(true);
  });

  it("hides non-cookie fields when cookie source is off", () => {
    const config = { cookie_source: "off" as const };

    expect(shouldShowProviderField(manualCookie, config)).toBe(false);
    expect(shouldShowProviderField(historyWindow, config)).toBe(false);
    expect(shouldShowProviderField(apiKey, config)).toBe(false);
    expect(shouldShowProviderField(cookieSource, config)).toBe(true);
  });

  it("always shows api key fields regardless of cookie source", () => {
    expect(shouldShowProviderField(apiKey, { cookie_source: "auto" })).toBe(true);
    expect(shouldShowProviderField(apiKey, { cookie_source: "manual" })).toBe(true);
  });
});
