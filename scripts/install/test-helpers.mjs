import { execFileSync } from "node:child_process";
import path from "node:path";

export const root = path.resolve(import.meta.dirname, "../..");
export const commonSh = path.join(root, "scripts/install/lib/common.sh");
export const linuxDepsSh = path.join(root, "scripts/install/lib/linux-deps.sh");
export const macosCliSh = path.join(root, "scripts/install/lib/macos-cli.sh");
export const homebrewTapSh = path.join(root, "scripts/install/lib/homebrew-tap.sh");

/** Prefer macOS /bin/bash when present so CI/dev catches bash 3.2 issues. */
export function bashBin() {
  if (process.platform === "darwin") {
    return "/bin/bash";
  }
  return "bash";
}

export function runBash(script, env = {}) {
  return execFileSync(bashBin(), ["-c", script], {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      COMMON_SH: commonSh,
      LINUX_DEPS_SH: linuxDepsSh,
      MACOS_CLI_SH: macosCliSh,
      HOMEBREW_TAP_SH: homebrewTapSh,
      ...env,
    },
  });
}

export function sourceCommon(call, env = {}) {
  return runBash(`set -euo pipefail\nsource "$COMMON_SH"\n${call}`, env).trim();
}

export function sourceLinuxDeps(call, env = {}) {
  return runBash(
    `set -euo pipefail\nsource "$COMMON_SH"\nsource "$LINUX_DEPS_SH"\n${call}`,
    env,
  ).trim();
}

export function sourceMacosCli(call, env = {}) {
  return runBash(`set -euo pipefail\nsource "$MACOS_CLI_SH"\n${call}`, env).trim();
}

export function bashSyntaxCheck(scriptPath) {
  execFileSync(bashBin(), ["-n", scriptPath], { encoding: "utf8" });
}

export function pwshBin() {
  for (const candidate of ["pwsh", "powershell"]) {
    try {
      execFileSync(candidate, ["-NoProfile", "-Command", "$PSVersionTable.PSVersion.Major"], {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "ignore"],
      });
      return candidate;
    } catch {
      // try next
    }
  }
  return null;
}

export function runPwsh(script, env = {}) {
  const shell = pwshBin();
  if (!shell) {
    throw new Error("PowerShell (pwsh or powershell) is not available");
  }
  return execFileSync(shell, ["-NoProfile", "-Command", script], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, ...env },
  });
}
