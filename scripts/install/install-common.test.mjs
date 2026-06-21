import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { describe, expect, it } from "vitest";

import { sourceCommon } from "./test-helpers.mjs";

function withReleaseFixture(releases, fn, env = {}) {
  const dir = mkdtempSync(path.join(tmpdir(), "mochi-install-test-"));
  const fixture = path.join(dir, "releases.json");
  writeFileSync(fixture, `${JSON.stringify(releases)}\n`);
  return fn(fixture, dir);
}

describe("install common.sh", () => {
  it("resolves unstable installs to the newest timestamped unstable prerelease", () => {
    withReleaseFixture(
      [
        {
          tag_name: "unstable",
          prerelease: true,
          draft: false,
          published_at: "2026-05-25T02:07:00Z",
        },
        {
          tag_name: "unstable-20260606.145138",
          prerelease: true,
          draft: false,
          published_at: "2026-06-06T14:53:53Z",
        },
        {
          tag_name: "unstable-20260606.150109",
          prerelease: true,
          draft: false,
          published_at: "2026-06-06T15:03:34Z",
        },
        {
          tag_name: "v0.2.2",
          prerelease: false,
          draft: false,
          published_at: "2026-06-06T14:26:43Z",
        },
      ],
      (fixture) => {
        const tag = sourceCommon(
          `mochi_curl_json() { cat "${fixture}"; }\nmochi_resolve_release_tag`,
          { MOCHI_INSTALL_UNSTABLE: "1" },
        );
        expect(tag).toBe("unstable-20260606.150109");
      },
    );
  });

  it("resolves stable installs to the newest non-prerelease release", () => {
    withReleaseFixture(
      [
        {
          tag_name: "v0.2.1",
          prerelease: false,
          draft: false,
          published_at: "2026-06-06T12:00:00Z",
        },
        {
          tag_name: "v0.2.2",
          prerelease: false,
          draft: false,
          published_at: "2026-06-06T14:26:43Z",
        },
      ],
      (fixture) => {
        const tag = sourceCommon(
          `mochi_curl_json() { cat "${fixture}"; }\nmochi_resolve_release_tag`,
        );
        expect(tag).toBe("v0.2.1");
      },
    );
  });

  it("honors explicit release tags and MOCHI_VERSION", () => {
    expect(sourceCommon("MOCHI_REQUESTED_TAG=v1.0.0; mochi_resolve_release_tag")).toBe("v1.0.0");
    expect(sourceCommon("mochi_resolve_release_tag", { MOCHI_VERSION: "v9.9.9" })).toBe("v9.9.9");
  });

  it("parses -i/--unstable and rejects unknown flags", () => {
    const unstable = sourceCommon(
      `MOCHI_INSTALL_SCRIPT_NAME=install.sh; mochi_parse_install_args -i; echo "$MOCHI_INSTALL_UNSTABLE"`,
    );
    expect(unstable).toBe("1");

    expect(() =>
      sourceCommon(`MOCHI_INSTALL_SCRIPT_NAME=install.sh; mochi_parse_install_args --wat`),
    ).toThrow();
  });

  it("picks the newest matching release asset by updated_at", () => {
    const release = {
      assets: [
        {
          name: "Mochi_0.2.0_amd64.AppImage",
          browser_download_url: "https://example.com/old.AppImage",
          updated_at: "2026-01-01T00:00:00Z",
        },
        {
          name: "Mochi_0.2.1_amd64.AppImage",
          browser_download_url: "https://example.com/new.AppImage",
          updated_at: "2026-06-01T00:00:00Z",
        },
      ],
    };
    const dir = mkdtempSync(path.join(tmpdir(), "mochi-release-json-"));
    const fixture = path.join(dir, "release.json");
    writeFileSync(fixture, `${JSON.stringify(release)}\n`);

    const url = sourceCommon(
      `release_json=$(cat "${fixture}"); mochi_pick_asset_url "$release_json" '\\.AppImage$'`,
    );
    expect(url).toBe("https://example.com/new.AppImage");
  });

  it("selects linux package kinds without relying on host package managers", () => {
    expect(sourceCommon("mochi_linux_package_kind", { MOCHI_TEST_LINUX_FAMILY: "debian" })).toBe(
      "deb",
    );
    expect(sourceCommon("mochi_linux_package_kind", { MOCHI_TEST_LINUX_FAMILY: "fedora" })).toBe(
      "rpm",
    );
    expect(sourceCommon("mochi_linux_package_kind", { MOCHI_TEST_LINUX_FAMILY: "generic" })).toBe(
      "appimage",
    );
    expect(sourceCommon("mochi_linux_package_kind", { MOCHI_PACKAGE: "deb" })).toBe("deb");
  });

  it("maps linux package kinds to asset patterns", () => {
    const patterns = sourceCommon(`mochi_linux_asset_patterns deb`);
    expect(patterns.split("\n")).toEqual(["\\.deb$", "_amd64\\.deb$"]);
  });
});
