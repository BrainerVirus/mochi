import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { bashSyntaxCheck, bashBin, sourceCommon } from "./test-helpers.mjs";

const installLinuxSh = path.join(import.meta.dirname, "install-linux.sh");

describe("install-linux.sh package selection", () => {
  it("uses deb/rpm/appimage patterns derived from package kind", () => {
    expect(sourceCommon(`mochi_linux_asset_patterns deb`).split("\n")).toEqual([
      "\\.deb$",
      "_amd64\\.deb$",
    ]);
    expect(sourceCommon(`mochi_linux_asset_patterns rpm`).split("\n")).toEqual([
      "\\.rpm$",
      "x86_64\\.rpm$",
    ]);
    expect(sourceCommon(`mochi_linux_asset_patterns appimage`).split("\n")).toEqual([
      "\\.AppImage$",
      "appimage",
    ]);
  });

  it("rejects unsupported package kinds", () => {
    expect(() => sourceCommon(`mochi_linux_asset_patterns snap`)).toThrow();
  });

  it("picks linux release assets for each package kind", () => {
    const release = {
      assets: [
        {
          name: "Mochi_0.2.1_amd64.deb",
          browser_download_url: "https://example.com/pkg.deb",
          updated_at: "2026-06-01T00:00:00Z",
        },
        {
          name: "Mochi_0.2.1_amd64.AppImage",
          browser_download_url: "https://example.com/pkg.AppImage",
          updated_at: "2026-06-01T00:00:00Z",
        },
      ],
    };
    const dir = mkdtempSync(path.join(tmpdir(), "mochi-linux-release-"));
    const fixture = path.join(dir, "release.json");
    writeFileSync(fixture, `${JSON.stringify(release)}\n`);

    const debUrl = sourceCommon(
      `release_json=$(cat "${fixture}"); mochi_pick_asset_url "$release_json" '\\.deb$' '_amd64\\.deb$'`,
    );
    expect(debUrl).toBe("https://example.com/pkg.deb");
  });
});

describe("install-linux.sh", () => {
  it(`parses under ${bashBin()}`, () => {
    expect(() => bashSyntaxCheck(installLinuxSh)).not.toThrow();
  });
});
