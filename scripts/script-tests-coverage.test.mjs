import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { bashBin, bashSyntaxCheck, pwshBin } from "./install/test-helpers.mjs";

const root = path.resolve(import.meta.dirname, "..");

function walkScripts(dir, files = []) {
  for (const entry of readdirSync(dir)) {
    const full = path.join(dir, entry);
    if (statSync(full).isDirectory()) {
      walkScripts(full, files);
      continue;
    }
    if (/\.(sh|ps1|mjs)$/.test(entry) && !/\.test\.(mjs|ts)$/.test(entry)) {
      files.push(full);
    }
  }
  return files;
}

const TESTED_BY = new Map([
  ["scripts/install/lib/common.sh", "scripts/install/install-common.test.mjs"],
  ["scripts/install/lib/linux-deps.sh", "scripts/install/linux-deps.test.mjs"],
  ["scripts/install/lib/homebrew-tap.sh", "scripts/install/homebrew-tap.test.mjs"],
  ["scripts/install/lib/macos-cli.sh", "scripts/install/install-macos.test.mjs"],
  ["scripts/install/lib/windows-install.ps1", "scripts/install/install-windows.test.mjs"],
  ["scripts/install/install-macos-brew.sh", "scripts/install/homebrew-tap.test.mjs"],
  ["scripts/install/setup-macos-brew-tap.sh", "scripts/install/homebrew-tap.test.mjs"],
  ["scripts/tsconfig-folder-resolver.cts", "scripts/vite-folder-resolver.test.ts"],
]);

const EXEMPT = new Set(["scripts/generate-icons.sh", "scripts/install/test-helpers.mjs"]);

function hasSiblingTest(scriptPath) {
  return [".test.mjs", ".test.ts"].some((suffix) => {
    const candidate = scriptPath.replace(/\.(sh|ps1|mjs)$/, suffix);
    try {
      return statSync(candidate).isFile();
    } catch {
      return false;
    }
  });
}

describe("script test coverage gate", () => {
  it("requires every scripts/* executable to be covered by a vitest suite", () => {
    const scripts = walkScripts(path.join(root, "scripts"));
    const missing = [];

    for (const scriptPath of scripts) {
      const rel = path.relative(root, scriptPath).split(path.sep).join("/");
      if (EXEMPT.has(rel)) continue;
      if (hasSiblingTest(scriptPath)) continue;
      if (TESTED_BY.has(rel)) continue;
      missing.push(rel);
    }

    expect(missing).toEqual([]);
  });
});

describe("bash install scripts", () => {
  const shellScripts = walkScripts(path.join(root, "scripts")).filter((file) =>
    file.endsWith(".sh"),
  );

  it.each(shellScripts.map((file) => [path.relative(root, file), file]))(
    "%s passes bash -n",
    (_label, scriptPath) => {
      expect(() => bashSyntaxCheck(scriptPath)).not.toThrow();
    },
  );
});

describe("powershell install scripts", () => {
  it("parses install-windows.ps1 when pwsh is available", () => {
    const shell = pwshBin();
    if (!shell) {
      return;
    }

    const installPs1 = path.join(root, "scripts/install/install-windows.ps1");
    const libPs1 = path.join(root, "scripts/install/lib/windows-install.ps1");
    const parse = (file) =>
      execFileSync(
        shell,
        [
          "-NoProfile",
          "-Command",
          `$e=$null; $t=$null; [void][System.Management.Automation.Language.Parser]::ParseFile('${file.replace(/'/g, "''")}', [ref]$t, [ref]$e); if ($e) { throw $e[0].Message }`,
        ],
        { encoding: "utf8" },
      );

    expect(() => parse(libPs1)).not.toThrow();
    expect(() => parse(installPs1)).not.toThrow();
  });
});
