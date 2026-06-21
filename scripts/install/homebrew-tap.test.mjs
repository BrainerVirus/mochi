import { execFileSync } from "node:child_process";
import {
  chmodSync,
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

const root = path.resolve(import.meta.dirname, "../..");
const libSh = path.join(root, "scripts/install/lib/homebrew-tap.sh");
const setupSh = path.join(root, "scripts/install/setup-macos-brew-tap.sh");
const installSh = path.join(root, "scripts/install/install-macos-brew.sh");

function runBash(script, env = {}) {
  return execFileSync("/bin/bash", ["-c", script], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, LIB: libSh, ...env },
  });
}

function tapRemoteEnv(remotes) {
  return Object.fromEntries(
    Object.entries(remotes).map(([tap, url]) => [
      `MOCHI_TEST_TAP_REMOTE_${tap.replace(/\//g, "_").replace(/-/g, "_").toLowerCase()}`,
      url,
    ]),
  );
}

function tapPlan({ taps = [], remotes = {}, repo = "BrainerVirus/mochi" }) {
  const tapList = taps.join("\n");
  return runBash(
    `source "$LIB"
tap_list=$(cat <<'TAPS'
${tapList}
TAPS
)
mochi_homebrew_tap_plan "$tap_list"`,
    {
      MOCHI_GITHUB_REPO: repo,
      ...tapRemoteEnv(remotes),
    },
  ).trim();
}

function isolatedToolPath(dir) {
  return `${dir}:/usr/bin:/bin`;
}

function createFakeBrew({ taps = [], remotes = {} } = {}) {
  const dir = mkdtempSync(path.join(tmpdir(), "mochi-brew-test-"));
  const logFile = path.join(dir, "brew.log");
  const tapsFile = path.join(dir, "taps.txt");
  const remotesDir = path.join(dir, "remotes");
  mkdirSync(remotesDir);

  writeFileSync(tapsFile, `${taps.join("\n")}\n`);
  for (const [tap, url] of Object.entries(remotes)) {
    writeFileSync(path.join(remotesDir, tap.replace(/\//g, "_").replace(/-/g, "_")), url);
  }

  const brewPath = path.join(dir, "brew");
  writeFileSync(
    brewPath,
    `#!/usr/bin/env bash
set -euo pipefail
echo "$*" >> "${logFile}"
case "$1" in
  tap)
    if [ "$#" -eq 1 ]; then
      cat "${tapsFile}" 2>/dev/null || true
      exit 0
    fi
    if [ "$2" = "--repo" ]; then
      key=$(printf '%s' "$3" | tr '/-' '_')
      if [ -f "${remotesDir}/$key" ]; then
        cat "${remotesDir}/$key"
      else
        printf '%s\\n' "${dir}/checkout"
      fi
      exit 0
    fi
    printf '%s\\n' "$2" >> "${tapsFile}"
    key=$(printf '%s' "$2" | tr '/-' '_')
    printf '%s' "$3" > "${remotesDir}/$key"
    ;;
  untap)
    tmp=$(mktemp)
    key=$(printf '%s' "$3" | tr '/-' '_')
    grep -Fxv "$3" "${tapsFile}" > "$tmp" || true
    mv "$tmp" "${tapsFile}"
    rm -f "${remotesDir}/$key"
    ;;
  install)
    exit 0
    ;;
esac
`,
  );
  chmodSync(brewPath, 0o755);

  return {
    dir,
    logFile,
    pathPrefix: isolatedToolPath(dir),
    readLog() {
      return existsSync(logFile) ? readFileSync(logFile, "utf8").trim() : "";
    },
  };
}

function createFakeCurl() {
  const dir = mkdtempSync(path.join(tmpdir(), "mochi-curl-test-"));
  const curlPath = path.join(dir, "curl");
  const commonSh = path.join(root, "scripts/install/lib/common.sh");
  const homebrewSh = path.join(root, "scripts/install/lib/homebrew-tap.sh");
  const setupShLocal = path.join(root, "scripts/install/setup-macos-brew-tap.sh");

  writeFileSync(
    curlPath,
    `#!/usr/bin/env bash
set -euo pipefail
url=""
out=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    -fsSL) shift ;;
    -o)
      out="$2"
      shift 2
      ;;
    *)
      url="$1"
      shift
      ;;
  esac
done
case "$url" in
  *scripts/install/lib/common.sh)
    cat "${commonSh}" > "$out"
    ;;
  *scripts/install/lib/homebrew-tap.sh)
    cat "${homebrewSh}" > "$out"
    ;;
  *scripts/install/setup-macos-brew-tap.sh)
    cat "${setupShLocal}"
    ;;
  *)
    echo "unexpected curl: $url" >&2
    exit 1
    ;;
esac
`,
  );
  chmodSync(curlPath, 0o755);

  return {
    pathPrefix: isolatedToolPath(dir),
  };
}

describe("homebrew tap plan", () => {
  it("matches tap names case-insensitively without bash 4 lowercase syntax", () => {
    const plan = tapPlan({
      taps: ["BrainerVirus/mochi"],
      remotes: {
        "BrainerVirus/mochi": "https://github.com/BrainerVirus/mochi",
      },
    });

    expect(plan).toContain("ready\tBrainerVirus/mochi\thttps://github.com/BrainerVirus/mochi");
  });

  it("removes stale dev taps that break brew update", () => {
    const plan = tapPlan({
      taps: ["test/mochi-tap", "brainervirus/mochi-install", "brainervirus/mochi"],
      remotes: {
        "brainervirus/mochi": "file:///Users/dev/mochi",
      },
    });

    expect(plan).toContain("remove-stale\ttest/mochi-tap");
    expect(plan).toContain("remove-stale\tBrainerVirus/mochi-install");
  });

  it("replaces a file:// tap with the explicit GitHub repo URL", () => {
    const plan = tapPlan({
      taps: ["brainervirus/mochi"],
      remotes: {
        "brainervirus/mochi": "file:///Users/dev/mochi",
      },
    });

    expect(plan).toContain(
      "replace\tBrainerVirus/mochi\tfile:///Users/dev/mochi\thttps://github.com/BrainerVirus/mochi",
    );
  });

  it("installs the tap from the main repo URL when it is missing", () => {
    const plan = tapPlan({ taps: [] });

    expect(plan).toBe("install\tBrainerVirus/mochi\thttps://github.com/BrainerVirus/mochi");
  });
});

describe("homebrew install cask ref", () => {
  it("uses the stable desktop cask token to avoid the official mochi cask", () => {
    const ref = runBash(`source "$LIB" && mochi_homebrew_install_cask_ref stable`).trim();

    expect(ref).toBe("BrainerVirus/mochi/mochi-desktop");
    expect(ref).not.toBe("mochi");
    expect(ref).not.toBe("BrainerVirus/mochi/mochi");
  });

  it("uses the unstable cask token", () => {
    const ref = runBash(`source "$LIB" && mochi_homebrew_install_cask_ref unstable`).trim();
    expect(ref).toBe("BrainerVirus/mochi/mochi-unstable");
  });
});

describe("packaged Homebrew casks", () => {
  it("ships mochi-desktop instead of colliding with homebrew/cask/mochi", () => {
    expect(existsSync(path.join(root, "Casks/mochi.rb"))).toBe(false);
    expect(readFileSync(path.join(root, "Casks/mochi-desktop.rb"), "utf8")).toContain(
      'cask "mochi-desktop" do',
    );
  });
});

describe("setup-macos-brew-tap.sh", () => {
  it("parses under macOS /bin/bash 3.2", () => {
    expect(() => execFileSync("/bin/bash", ["-n", setupSh], { encoding: "utf8" })).not.toThrow();
  });

  it("runs under /bin/bash and taps the GitHub repo explicitly", () => {
    const fakeBrew = createFakeBrew({
      taps: ["test/mochi-tap", "brainervirus/mochi"],
      remotes: {
        "brainervirus/mochi": "file:///Users/dev/mochi",
      },
    });

    const output = execFileSync("/bin/bash", [setupSh], {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: fakeBrew.pathPrefix,
        MOCHI_TEST_TAP_REMOTE_brainervirus_mochi: "file:///Users/dev/mochi",
      },
    });

    const log = fakeBrew.readLog();
    expect(output).toContain("Removing stale tap test/mochi-tap");
    expect(output).toContain("Replacing tap BrainerVirus/mochi");
    expect(log).toContain("untap --force BrainerVirus/mochi");
    expect(log).toContain("tap BrainerVirus/mochi https://github.com/BrainerVirus/mochi");
  });
});

describe("install-macos-brew.sh", () => {
  it("parses under macOS /bin/bash 3.2", () => {
    expect(() => execFileSync("/bin/bash", ["-n", installSh], { encoding: "utf8" })).not.toThrow();
  });

  it("installs the fully qualified stable cask after tapping GitHub", () => {
    const fakeBrew = createFakeBrew({ taps: [] });
    execFileSync("/bin/bash", [installSh], {
      cwd: path.dirname(installSh),
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: fakeBrew.pathPrefix,
      },
    });

    const log = fakeBrew.readLog();
    expect(log).toContain("tap BrainerVirus/mochi https://github.com/BrainerVirus/mochi");
    expect(log).toContain("install --cask BrainerVirus/mochi/mochi-desktop --force --no-quarantine");
    expect(log).not.toMatch(/\binstall --cask mochi\b/);
  });

  it("installs the fully qualified unstable cask", () => {
    const fakeBrew = createFakeBrew({ taps: [] });
    execFileSync("/bin/bash", [installSh, "-i"], {
      cwd: path.dirname(installSh),
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: fakeBrew.pathPrefix,
      },
    });

    expect(fakeBrew.readLog()).toContain(
      "install --cask BrainerVirus/mochi/mochi-unstable --force --no-quarantine",
    );
  });

  it("works when piped to bash from an unrelated working directory", () => {
    const fakeBrew = createFakeBrew({ taps: [] });
    const fakeCurl = createFakeCurl();
    const pipedEnv = {
      HOME: process.env.HOME ?? tmpdir(),
      PATH: `${fakeCurl.pathPrefix}:${fakeBrew.pathPrefix}`,
      TMPDIR: process.env.TMPDIR ?? tmpdir(),
      MOCHI_GITHUB_REPO: "BrainerVirus/mochi",
      MOCHI_INSTALL_REF: "main",
    };
    const wrongCwd = mkdtempSync(path.join(tmpdir(), "mochi-piped-install-"));

    execFileSync("/bin/bash", ["-s"], {
      input: readFileSync(installSh),
      cwd: wrongCwd,
      encoding: "utf8",
      env: pipedEnv,
    });

    const log = fakeBrew.readLog();
    expect(log).toContain("tap BrainerVirus/mochi https://github.com/BrainerVirus/mochi");
    expect(log).toContain("install --cask BrainerVirus/mochi/mochi-desktop --force --no-quarantine");
  });
});
