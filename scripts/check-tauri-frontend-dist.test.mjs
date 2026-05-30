import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { describe, expect, it } from "vitest";

import { validateTauriFrontendDist } from "./check-tauri-frontend-dist.mjs";

function writeShellFixture(shell) {
  const root = mkdtempSync(join(tmpdir(), "mochi-shell-"));
  const publicDir = join(root, "dist");
  mkdirSync(join(publicDir, "assets"), { recursive: true });
  writeFileSync(join(publicDir, "assets", "index-test.js"), "console.log('ok');");
  writeFileSync(join(publicDir, "assets", "index-test.css"), "body{}");
  writeFileSync(join(publicDir, "index.html"), shell);
  return publicDir;
}

describe("validateTauriFrontendDist", () => {
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

    expect(validateTauriFrontendDist(publicDir)).toEqual([]);
  });

  it("rejects framework hydration documents without the React mount root", () => {
    const publicDir = writeShellFixture(`<!doctype html>
<html>
  <body>
    <script class="$tsr">self.$_TSR = {}</script>
    <script type="module" src="/./assets/index-test.js"></script>
  </body>
</html>`);

    expect(validateTauriFrontendDist(publicDir)).toContain(
      "index.html must include the React mount target div#root.",
    );
  });

  it("rejects root-absolute bundled asset references", () => {
    const publicDir = writeShellFixture(`<!doctype html>
<html>
  <body>
    <div id="root"></div>
    <script type="module" src="/assets/index-test.js"></script>
  </body>
</html>`);

    expect(validateTauriFrontendDist(publicDir)).toContain(
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

    expect(validateTauriFrontendDist(publicDir)).toContain(
      "index.html references missing asset: ./assets/missing.js",
    );
  });

  it("rejects client chunks that preload bundled assets from root-absolute paths", () => {
    const publicDir = writeShellFixture(`<!doctype html>
<html>
  <body>
    <div id="root"></div>
    <script type="module" src="./assets/index-test.js"></script>
  </body>
</html>`);
    writeFileSync(
      join(publicDir, "assets", "index-test.js"),
      'const preload = (asset) => "/" + asset;',
    );

    expect(validateTauriFrontendDist(publicDir)).toContain(
      "client chunks must not construct root-absolute /assets URLs for packaged Tauri webviews.",
    );
  });
});
