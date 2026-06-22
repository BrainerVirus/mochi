// @vitest-environment happy-dom
import { waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { queryKeys } from "@/lib/query/keys";
import { DEFAULT_MOCHI_SETTINGS } from "@/lib/schemas/settings";
import { logFrontendDebug, reportFrontendError } from "@/lib/tauri/diagnostics";

import {
  handleSettingsChangedEvent,
  parseSettingsChangedPayload,
  settingsCacheMatches,
} from "./use-tray-events";

vi.mock("@/lib/tauri/diagnostics", () => ({
  logFrontendDebug: vi.fn<(scope: string, message: string) => void>(),
  reportFrontendError: vi.fn<() => Promise<void>>(() => Promise.resolve()),
}));

beforeEach(() => {
  vi.mocked(logFrontendDebug).mockClear();
  vi.mocked(reportFrontendError).mockClear();
});

describe("settingsCacheMatches", () => {
  it("matches settings when provider config keys are ordered differently", () => {
    const cached = {
      ...DEFAULT_MOCHI_SETTINGS,
      provider_configs: {
        codex: { api_key: "cached" },
        claude: { api_key: "cached" },
      },
    };
    const next = {
      ...DEFAULT_MOCHI_SETTINGS,
      provider_configs: {
        claude: { api_key: "cached" },
        codex: { api_key: "cached" },
      },
    };

    expect(settingsCacheMatches(cached, next)).toBe(true);
  });

  it("detects enabled provider list changes", () => {
    expect(
      settingsCacheMatches(
        { ...DEFAULT_MOCHI_SETTINGS, enabled_providers: ["codex", "gemini"] },
        { ...DEFAULT_MOCHI_SETTINGS, enabled_providers: ["codex"] },
      ),
    ).toBe(false);
  });
});

describe("parseSettingsChangedPayload", () => {
  it("returns parsed settings for valid payloads", () => {
    expect(parseSettingsChangedPayload(DEFAULT_MOCHI_SETTINGS)).toEqual(DEFAULT_MOCHI_SETTINGS);
  });

  it("returns null for invalid payloads", () => {
    expect(parseSettingsChangedPayload({ enabled_providers: ["not-a-provider"] })).toBeNull();
    expect(parseSettingsChangedPayload(null)).toBeNull();
  });
});

describe("handleSettingsChangedEvent", () => {
  it("reconciles valid settings-changed payloads", async () => {
    const reconcile = vi.fn<() => Promise<void>>(() => Promise.resolve());
    const queryClient = {
      setQueryData: () => undefined,
      getQueryData: () => undefined,
      invalidateQueries: () => Promise.resolve(),
    };

    handleSettingsChangedEvent(DEFAULT_MOCHI_SETTINGS, queryClient, reconcile);

    await waitFor(() => {
      expect(reconcile).toHaveBeenCalledWith(queryClient, DEFAULT_MOCHI_SETTINGS);
    });
  });

  it("logs and skips invalid settings-changed payloads", () => {
    const reconcile = vi.fn<() => Promise<void>>(() => Promise.resolve());
    const logInvalid = vi.fn<(message: string) => void>(() => undefined);
    const queryClient = {
      setQueryData: () => undefined,
      getQueryData: () => undefined,
      invalidateQueries: () => Promise.resolve(),
    };

    handleSettingsChangedEvent(
      { enabled_providers: ["not-a-provider"] },
      queryClient,
      reconcile,
      logInvalid,
    );

    expect(logInvalid).toHaveBeenCalledWith("settings-changed payload failed validation");
    expect(reconcile).not.toHaveBeenCalled();
  });

  it("skips reconcile when settings cache already matches the payload", () => {
    const reconcile = vi.fn<() => Promise<void>>(() => Promise.resolve());
    const queryClient = {
      setQueryData: () => undefined,
      getQueryData: (queryKey: readonly unknown[]) =>
        queryKey === queryKeys.settings ? DEFAULT_MOCHI_SETTINGS : undefined,
      invalidateQueries: () => Promise.resolve(),
    };

    handleSettingsChangedEvent(DEFAULT_MOCHI_SETTINGS, queryClient, reconcile);

    expect(reconcile).not.toHaveBeenCalled();
  });

  it("swallows reconcile failures after logging debug and diagnostics output", async () => {
    const reconcile = vi.fn<() => Promise<void>>(() => Promise.reject(new Error("sync failed")));
    const queryClient = {
      setQueryData: () => undefined,
      getQueryData: () => undefined,
      invalidateQueries: () => Promise.resolve(),
    };

    handleSettingsChangedEvent(DEFAULT_MOCHI_SETTINGS, queryClient, reconcile);

    await waitFor(() => {
      expect(reconcile).toHaveBeenCalled();
      expect(logFrontendDebug).toHaveBeenCalledWith("settings-reconcile", "sync failed");
      expect(reportFrontendError).toHaveBeenCalledWith("sync failed", "debug:settings-reconcile");
    });
  });
});
