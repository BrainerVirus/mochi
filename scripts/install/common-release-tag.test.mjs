import { execFileSync } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

const root = path.resolve(import.meta.dirname, "../..");
const commonSh = path.join(root, "scripts/install/lib/common.sh");

function resolveTag({ releases, env = {} }) {
  const dir = mkdtempSync(path.join(tmpdir(), "mochi-install-test-"));
  const fixture = path.join(dir, "releases.json");
  writeFileSync(fixture, `${JSON.stringify(releases)}\n`);

  const script = `
    set -euo pipefail
    source "$COMMON_SH"
    mochi_curl_json() { cat "$RELEASES_JSON"; }
    mochi_resolve_release_tag
  `;

  return execFileSync("bash", ["-c", script], {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      COMMON_SH: commonSh,
      RELEASES_JSON: fixture,
      ...env,
    },
  }).trim();
}

describe("install release tag resolver", () => {
  it("resolves unstable installs to the newest timestamped unstable prerelease", () => {
    const tag = resolveTag({
      env: { MOCHI_INSTALL_UNSTABLE: "1" },
      releases: [
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
    });

    expect(tag).toBe("unstable-20260606.150109");
  });
});
