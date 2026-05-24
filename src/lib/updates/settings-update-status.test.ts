import { describe, expect, it } from "vitest";

import {
  resolveSettingsUpdateStatusLabel,
  shouldShowSettingsInstallButton,
} from "@/lib/updates/settings-update-status";

describe("resolveSettingsUpdateStatusLabel", () => {
  it("reports idle up-to-date state", () => {
    expect(
      resolveSettingsUpdateStatusLabel({
        updateAvailable: false,
        version: null,
        isFetching: false,
        installPhase: "idle",
        installPending: false,
        installError: null,
      }),
    ).toBe("You're up to date");
  });

  it("reports available update with version", () => {
    expect(
      resolveSettingsUpdateStatusLabel({
        updateAvailable: true,
        version: "0.2.0",
        isFetching: false,
        installPhase: "idle",
        installPending: false,
        installError: null,
      }),
    ).toBe("Update available (v0.2.0)");
  });

  it("prefers install errors over other status text", () => {
    expect(
      resolveSettingsUpdateStatusLabel({
        updateAvailable: true,
        version: "0.2.0",
        isFetching: false,
        installPhase: "error",
        installPending: false,
        installError: "Download failed",
      }),
    ).toBe("Download failed");
  });
});

describe("shouldShowSettingsInstallButton", () => {
  it("shows install when an update is available and idle", () => {
    expect(
      shouldShowSettingsInstallButton({
        updateAvailable: true,
        installPending: false,
      }),
    ).toBe(true);
  });

  it("hides install while an update is already running", () => {
    expect(
      shouldShowSettingsInstallButton({
        updateAvailable: true,
        installPending: true,
      }),
    ).toBe(false);
  });
});
