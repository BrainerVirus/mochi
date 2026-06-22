import { execFileSync } from "node:child_process";
import { chmodSync, mkdtempSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { bashSyntaxCheck, bashBin, sourceMacosCli } from "./test-helpers.mjs";

const installMacosSh = path.join(import.meta.dirname, "install-macos.sh");

function writeFakeSudo(root) {
  const fakeSudo = path.join(root, "fake-sudo.sh");
  writeFileSync(
    fakeSudo,
    `#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == "mkdir" ]]; then
  shift
  mkdir -p "$@"
  for arg in "$@"; do
    [[ "$arg" == -* ]] && continue
    chmod u+rwx "$arg" 2>/dev/null || true
  done
  exit 0
fi
if [[ "$1" == "ln" ]]; then
  link_path="\${@: -1}"
  chmod u+rwx "$(dirname "$link_path")" 2>/dev/null || true
fi
exec "$@"
`,
  );
  chmodSync(fakeSudo, 0o755);
  return fakeSudo;
}

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

  it("uses sudo for the system path when an unprivileged write fails", () => {
    const root = mkdtempSync(path.join(tmpdir(), "mochi-macos-cli-"));
    const app = path.join(root, "Mochi.app");
    const binDir = path.join(app, "Contents", "MacOS");
    const systemBin = path.join(root, "system-bin");
    mkdirSync(binDir, { recursive: true });
    mkdirSync(systemBin);
    chmodSync(systemBin, 0o555);
    const binary = path.join(binDir, "mochi");
    writeFileSync(binary, "#!/bin/sh\nexit 0\n");
    chmodSync(binary, 0o755);
    const systemLink = path.join(systemBin, "mochi");

    try {
      const output = sourceMacosCli(`mochi_install_cli_link "${app}"`, {
        HOME: path.join(root, "home"),
        MOCHI_CLI_USR_LOCAL: systemLink,
        MOCHI_CLI_SUDO: writeFakeSudo(root),
      });

      expect(output).toContain("administrator password required");
      expect(output).toContain(`Installed CLI command to ${systemLink}`);
    } finally {
      chmodSync(systemBin, 0o755);
    }
  });

  it("falls back to ~/.local/bin and configures PATH when sudo is unavailable", () => {
    const root = mkdtempSync(path.join(tmpdir(), "mochi-macos-cli-"));
    const app = path.join(root, "Mochi.app");
    const binDir = path.join(app, "Contents", "MacOS");
    const blockedUsrLocal = path.join(root, "blocked-usr-local");
    writeFileSync(blockedUsrLocal, "not a directory");
    const home = path.join(root, "home");
    const userBin = path.join(home, ".local", "bin");
    const zprofile = path.join(home, ".zprofile");
    mkdirSync(binDir, { recursive: true });
    const binary = path.join(binDir, "mochi");
    writeFileSync(binary, "#!/bin/sh\nexit 0\n");
    chmodSync(binary, 0o755);

    const output = sourceMacosCli(`mochi_install_cli_link "${app}"`, {
      HOME: home,
      MOCHI_CLI_USR_LOCAL: path.join(blockedUsrLocal, "mochi"),
      MOCHI_CLI_SUDO: "/usr/bin/false",
    });

    expect(output).toContain(`Installed CLI command to ${path.join(userBin, "mochi")}`);
    expect(output).toContain(`Configured PATH in ${zprofile}`);
    expect(readFileSync(zprofile, "utf8")).toContain(userBin);
  });

  it("dies when no install location is writable", () => {
    const root = mkdtempSync(path.join(tmpdir(), "mochi-macos-cli-"));
    const app = path.join(root, "Mochi.app");
    const binDir = path.join(app, "Contents", "MacOS");
    mkdirSync(binDir, { recursive: true });
    const binary = path.join(binDir, "mochi");
    writeFileSync(binary, "#!/bin/sh\nexit 0\n");
    chmodSync(binary, 0o755);

    const blockedParent = path.join(root, "blocked-parent");
    writeFileSync(blockedParent, "not a directory");
    const linkPath = path.join(blockedParent, "mochi");
    const blockedHome = path.join(root, "blocked-home");
    writeFileSync(blockedHome, "not a directory");

    expect(() =>
      sourceMacosCli(`mochi_install_cli_link "${app}"`, {
        MOCHI_CLI_LINK: linkPath,
        HOME: blockedHome,
        MOCHI_CLI_SUDO: "/usr/bin/false",
      }),
    ).toThrow(/could not install CLI to/);
  });

  it("skips linking when the bundle binary is missing", () => {
    const root = mkdtempSync(path.join(tmpdir(), "mochi-macos-cli-"));
    const app = path.join(root, "Mochi.app");
    mkdirSync(app, { recursive: true });

    const output = sourceMacosCli(`mochi_install_cli_link "${app}"`);
    expect(output).toContain("Skipping CLI link");
  });

  it("no-ops when the app bundle is missing", () => {
    expect(() =>
      sourceMacosCli('mochi_clear_macos_app_quarantine "/no/such/Mochi.app"'),
    ).not.toThrow();
  });

  it.skipIf(process.platform !== "darwin")("removes quarantine from an app bundle", () => {
    const root = mkdtempSync(path.join(tmpdir(), "mochi-macos-quarantine-"));
    const app = path.join(root, "Mochi.app");
    mkdirSync(app, { recursive: true });
    execFileSync("xattr", ["-w", "com.apple.quarantine", "0081;00000000;Safari;", app]);

    const output = sourceMacosCli(`mochi_clear_macos_app_quarantine "${app}"`);
    expect(output).toContain(`Removed macOS quarantine from ${app}`);

    expect(() => execFileSync("xattr", ["-p", "com.apple.quarantine", app])).toThrow();
  });
});

describe("install-macos.sh", () => {
  it(`parses under ${bashBin()}`, () => {
    expect(() => bashSyntaxCheck(installMacosSh)).not.toThrow();
  });
});
