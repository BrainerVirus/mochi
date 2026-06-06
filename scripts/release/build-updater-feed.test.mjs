import { describe, expect, it } from "vitest";

import {
  buildUpdaterFeedEntries,
  endpointToPlatformKey,
  supportedRecoveryVersions,
} from "./build-updater-feed.mjs";

describe("updater feed builder", () => {
  it("maps endpoint target and arch to tauri platform keys", () => {
    expect(endpointToPlatformKey("darwin", "aarch64")).toBe("darwin-aarch64");
    expect(endpointToPlatformKey("darwin", "x86_64")).toBe("darwin-x86_64");
    expect(endpointToPlatformKey("linux", "x86_64")).toBe("linux-x86_64");
    expect(endpointToPlatformKey("windows", "x86_64")).toBe("windows-x86_64");
  });

  it("backfills minimum recovery versions", () => {
    expect(supportedRecoveryVersions(["0.2.1"])).toEqual(["0.1.7", "0.2.0", "0.2.1"]);
  });

  it("creates stable and unstable entries for every platform/version pair", () => {
    const entries = buildUpdaterFeedEntries({
      versions: ["0.1.7", "0.2.0"],
      channels: ["stable", "unstable"],
      latestVersion: "0.2.1",
      notes: "### What's changed\n- Fix updater",
      pubDate: "2026-06-06T12:34:56.000Z",
      artifacts: {
        "darwin-aarch64": {
          url: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1/Mochi_aarch64.app.tar.gz",
          signature: "sig-a",
        },
        "darwin-x86_64": {
          url: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1/Mochi_x64.app.tar.gz",
          signature: "sig-m",
        },
        "linux-x86_64": {
          url: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1/Mochi_0.2.1_amd64.AppImage.tar.gz",
          signature: "sig-l",
        },
        "windows-x86_64": {
          url: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1/Mochi_0.2.1_x64-setup.nsis.zip",
          signature: "sig-w",
        },
      },
    });

    expect(entries).toContainEqual(
      expect.objectContaining({
        path: "updates/darwin/aarch64/0.1.7/stable.json",
      }),
    );
    expect(entries).toContainEqual(
      expect.objectContaining({
        path: "updates/windows/x86_64/0.2.0/unstable.json",
      }),
    );
  });
});
