import { chmodSync, mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { bashSyntaxCheck, bashBin, sourceMacosCli } from "./test-helpers.mjs";

const installMacosSh = path.join(import.meta.dirname, "install-macos.sh");

describe("macOS CLI link helper", () => {
  it("creates a symlink when the app bundle binary is executable", () => {
    const root = mkdtempSync(path.join(tmpdir(), "mochi-macos-cli-"));
    const app = path.join(root, "Mochi.app");
    const binDir = path.join(app, "Contents", "MacOS");
    const linkDir = path.join(root, "bin");
    mkdirSync(binDir, { recursive: true });
    const binary = path.join(binDir, "mochi");
    writeFileSync(binary, "#!/bin/sh\nexit 0\n");
    chmodSync(binary, 0o755);

    const output = sourceMacosCli(`mochi_install_cli_link "${app}"`, {
      MOCHI_CLI_LINK: path.join(linkDir, "mochi"),
    });

    expect(output).toContain(`Installed CLI command to ${path.join(linkDir, "mochi")}`);
  });

  it("prints sudo fallback when the link directory is not writable", () => {
    const root = mkdtempSync(path.join(tmpdir(), "mochi-macos-cli-"));
    const app = path.join(root, "Mochi.app");
    const binDir = path.join(app, "Contents", "MacOS");
    mkdirSync(binDir, { recursive: true });
    const binary = path.join(binDir, "mochi");
    writeFileSync(binary, "#!/bin/sh\nexit 0\n");
    chmodSync(binary, 0o755);

    const output = sourceMacosCli(`mochi_install_cli_link "${app}"`, {
      MOCHI_CLI_LINK: "/usr/local/bin/mochi-test-not-writable",
    });

    expect(output).toContain("Could not write /usr/local/bin/mochi-test-not-writable");
    expect(output).toContain("sudo ln -sf");
    expect(output).toContain("Contents/MacOS/mochi");
  });

  it("skips linking when the bundle binary is missing", () => {
    const root = mkdtempSync(path.join(tmpdir(), "mochi-macos-cli-"));
    const app = path.join(root, "Mochi.app");
    mkdirSync(app, { recursive: true });

    const output = sourceMacosCli(`mochi_install_cli_link "${app}"`);
    expect(output).toContain("Skipping CLI link");
  });
});

describe("install-macos.sh", () => {
  it(`parses under ${bashBin()}`, () => {
    expect(() => bashSyntaxCheck(installMacosSh)).not.toThrow();
  });
});
