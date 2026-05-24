import { describe, expect, it } from "vitest";

import type { ProviderCatalogEntry } from "@/lib/schemas/provider-catalog";

import { shouldShowProviderField } from "./provider-field-visibility";

type SettingsField = ProviderCatalogEntry["settingsFields"][number];

function entryWithFields(settingsFields: SettingsField[]): ProviderCatalogEntry {
  return {
    id: "codex",
    displayName: "Codex",
    implementationStatus: "done",
    strategies: [],
    authRequirements: [],
    settingsFields,
  };
}

const cookieProviderEntry = entryWithFields([
  { key: "cookie_source", label: "Cookie source", kind: "cookie-source" },
  { key: "manual_cookie", label: "Manual cookie", kind: "manual-cookie" },
  {
    key: "history_window_days",
    label: "Session cost history window",
    kind: "history-window",
  },
]);

const opencodeGoEntry = entryWithFields([
  { key: "cookie_source", label: "Cookie source", kind: "cookie-source" },
  { key: "manual_cookie", label: "Manual cookie header", kind: "manual-cookie" },
  { key: "token_accounts", label: "Session tokens", kind: "token-account" },
  {
    key: "workspace_id",
    label: "Workspace ID (optional)",
    kind: "region-host",
  },
]);

const copilotEntry = entryWithFields([
  { key: "token_accounts", label: "Token accounts", kind: "token-account" },
]);

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
const sessionTokens: SettingsField = {
  key: "token_accounts",
  label: "Session tokens",
  kind: "token-account",
};
const workspaceId: SettingsField = {
  key: "workspace_id",
  label: "Workspace ID (optional)",
  kind: "region-host",
};

describe("shouldShowProviderField", () => {
  it("hides manual cookie and history window when cookie source is auto", () => {
    const config = { cookie_source: "auto" as const };

    expect(shouldShowProviderField(manualCookie, config, cookieProviderEntry)).toBe(false);
    expect(shouldShowProviderField(historyWindow, config, cookieProviderEntry)).toBe(false);
    expect(shouldShowProviderField(cookieSource, config, cookieProviderEntry)).toBe(true);
  });

  it("shows manual fields when cookie source is manual", () => {
    const config = { cookie_source: "manual" as const };

    expect(shouldShowProviderField(manualCookie, config, cookieProviderEntry)).toBe(true);
    expect(shouldShowProviderField(historyWindow, config, cookieProviderEntry)).toBe(true);
  });

  it("hides non-cookie fields when cookie source is off", () => {
    const config = { cookie_source: "off" as const };

    expect(shouldShowProviderField(manualCookie, config, cookieProviderEntry)).toBe(false);
    expect(shouldShowProviderField(historyWindow, config, cookieProviderEntry)).toBe(false);
    expect(shouldShowProviderField(apiKey, config, cookieProviderEntry)).toBe(false);
    expect(shouldShowProviderField(cookieSource, config, cookieProviderEntry)).toBe(true);
  });

  it("always shows api key fields regardless of cookie source", () => {
    const entry = entryWithFields([{ key: "api_key", label: "API key", kind: "api-key" }]);

    expect(shouldShowProviderField(apiKey, { cookie_source: "auto" }, entry)).toBe(true);
    expect(shouldShowProviderField(apiKey, { cookie_source: "manual" }, entry)).toBe(true);
  });

  it("hides OpenCode Go session tokens and workspace id when cookie source is auto", () => {
    const config = { cookie_source: "auto" as const };

    expect(shouldShowProviderField(sessionTokens, config, opencodeGoEntry)).toBe(false);
    expect(shouldShowProviderField(workspaceId, config, opencodeGoEntry)).toBe(false);
    expect(shouldShowProviderField(manualCookie, config, opencodeGoEntry)).toBe(false);
    expect(shouldShowProviderField(cookieSource, config, opencodeGoEntry)).toBe(true);
  });

  it("shows OpenCode Go manual fields when cookie source is manual", () => {
    const config = { cookie_source: "manual" as const };

    expect(shouldShowProviderField(sessionTokens, config, opencodeGoEntry)).toBe(true);
    expect(shouldShowProviderField(workspaceId, config, opencodeGoEntry)).toBe(true);
    expect(shouldShowProviderField(manualCookie, config, opencodeGoEntry)).toBe(true);
  });

  it("keeps Copilot token accounts visible without cookie source settings", () => {
    const config = { cookie_source: "auto" as const };

    expect(shouldShowProviderField(sessionTokens, config, copilotEntry)).toBe(true);
  });
});
