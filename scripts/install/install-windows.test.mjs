import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { pwshBin, runPwsh } from "./test-helpers.mjs";

const libPs1 = path.join(import.meta.dirname, "lib/windows-install.ps1");
const installPs1 = path.join(import.meta.dirname, "install-windows.ps1");

const pwsh = pwshBin();
const describePwsh = pwsh ? describe : describe.skip;

function withReleaseFixture(releases, fn) {
  const dir = mkdtempSync(path.join(tmpdir(), "mochi-windows-test-"));
  const fixture = path.join(dir, "releases.json");
  writeFileSync(fixture, `${JSON.stringify(releases)}\n`);
  return fn(fixture);
}

function invokeLib(call, env = {}) {
  return runPwsh(
    `$ErrorActionPreference = 'Stop'; . '${libPs1.replace(/'/g, "''")}'; ${call}`,
    env,
  ).trim();
}

function parsePs1(filePath) {
  runPwsh(
    `$e=$null; $t=$null; [void][System.Management.Automation.Language.Parser]::ParseFile('${filePath.replace(/'/g, "''")}', [ref]$t, [ref]$e); if ($e) { throw $e[0].Message }`,
  );
}

describePwsh("windows-install.ps1", () => {
  it("parses under PowerShell", () => {
    expect(() => parsePs1(libPs1)).not.toThrow();
    expect(() => parsePs1(installPs1)).not.toThrow();
  });

  it("resolves unstable installs to the newest timestamped prerelease", () => {
    withReleaseFixture(
      [
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
        const tag = invokeLib(
          "Resolve-MochiReleaseTag -ReleaseTag '' -Unstable:$true -ApiBase 'https://api.github.com/repos/BrainerVirus/mochi'",
          { MOCHI_TEST_RELEASES_JSON: fixture },
        );
        expect(tag).toBe("unstable-20260606.150109");
      },
    );
  });

  it("resolves stable installs to the first non-prerelease release", () => {
    withReleaseFixture(
      [
        {
          tag_name: "v0.2.2",
          prerelease: false,
          draft: false,
          published_at: "2026-06-06T14:26:43Z",
        },
        {
          tag_name: "unstable-20260606.150109",
          prerelease: true,
          draft: false,
          published_at: "2026-06-06T15:03:34Z",
        },
      ],
      (fixture) => {
        const tag = invokeLib(
          "Resolve-MochiReleaseTag -ReleaseTag '' -Unstable:$false -ApiBase 'https://api.github.com/repos/BrainerVirus/mochi'",
          { MOCHI_TEST_RELEASES_JSON: fixture },
        );
        expect(tag).toBe("v0.2.2");
      },
    );
  });

  it("prefers MSI assets and falls back to setup EXE", () => {
    const release = {
      tag_name: "v0.2.2",
      assets: [
        {
          name: "Mochi_0.2.2_x64-setup.exe",
          browser_download_url: "https://example.com/setup.exe",
          updated_at: "2026-06-06T14:26:43Z",
        },
        {
          name: "Mochi_0.2.2_x64_en-US.msi",
          browser_download_url: "https://example.com/setup.msi",
          updated_at: "2026-06-06T15:00:00Z",
        },
      ],
    };

    const msi = invokeLib(
      `$release = '${JSON.stringify(release)}' | ConvertFrom-Json; (Resolve-MochiWindowsAsset -Release $release -Package 'auto').Asset.browser_download_url`,
    );
    expect(msi).toBe("https://example.com/setup.msi");

    const exeOnlyRelease = {
      tag_name: "v0.2.2",
      assets: [
        {
          name: "Mochi_0.2.2_x64-setup.exe",
          browser_download_url: "https://example.com/setup.exe",
          updated_at: "2026-06-06T14:26:43Z",
        },
      ],
    };
    const exe = invokeLib(
      `$release = '${JSON.stringify(exeOnlyRelease)}' | ConvertFrom-Json; (Resolve-MochiWindowsAsset -Release $release -Package 'auto').Asset.name`,
    );
    expect(exe).toBe("Mochi_0.2.2_x64-setup.exe");
  });
});

if (!pwsh) {
  describe("install-windows.ps1", () => {
    it.skip("requires pwsh locally; ubuntu CI runners provide it", () => {});
  });
}
