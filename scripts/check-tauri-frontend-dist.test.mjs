import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { describe, expect, it } from "vitest";

import { validateTauriShell } from "./check-tauri-frontend-dist.mjs";

function writeShellFixture(shell) {
  const root = mkdtempSync(join(tmpdir(), "mochi-shell-"));
  const publicDir = join(root, ".output", "public");
  mkdirSync(join(publicDir, "assets"), { recursive: true });
  writeFileSync(join(publicDir, "assets", "index-test.js"), "console.log('ok');");
  writeFileSync(join(publicDir, "assets", "index-test.css"), "body{}");
  writeFileSync(join(publicDir, "index.html"), shell);
  return publicDir;
}

describe("validateTauriShell", () => {
  it("accepts relative bundled asset references", () => {
    const publicDir = writeShellFixture(`<!doctype html>
<html>
  <head>
    <link rel="stylesheet" href="./assets/index-test.css" />
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="./assets/index-test.js"></script>
  </body>
</html>`);

    expect(validateTauriShell(publicDir)).toEqual([]);
  });

  it("rejects root-absolute bundled asset references", () => {
    const publicDir = writeShellFixture(`<!doctype html>
<html>
  <body>
    <div id="root"></div>
    <script type="module" src="/assets/index-test.js"></script>
  </body>
</html>`);

    expect(validateTauriShell(publicDir)).toContain(
      "index.html must use relative asset URLs for Tauri packaged webviews.",
    );
  });

  it("rejects referenced assets that are missing from the dist", () => {
    const publicDir = writeShellFixture(`<!doctype html>
<html>
  <body>
    <div id="root"></div>
    <script type="module" src="./assets/missing.js"></script>
  </body>
</html>`);

    expect(validateTauriShell(publicDir)).toContain(
      "index.html references missing asset: ./assets/missing.js",
    );
  });
});
