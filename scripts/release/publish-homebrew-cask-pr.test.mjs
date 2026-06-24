import { execFileSync } from "node:child_process";
import { chmodSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { bashBin, bashSyntaxCheck } from "../install/test-helpers.mjs";

const script = path.join(import.meta.dirname, "publish-homebrew-cask-pr.sh");
const temporaryDirectories = [];

function writeExecutable(file, source) {
  writeFileSync(file, source);
  chmodSync(file, 0o755);
}

function createFixture({
  closedPr = "",
  createPrFailure = false,
  hasChanges = true,
  openPr = "",
  remoteSha = "",
} = {}) {
  const directory = mkdtempSync(path.join(tmpdir(), "mochi-homebrew-publish-"));
  const bin = path.join(directory, "bin");
  const commandLog = path.join(directory, "commands.log");
  mkdirSync(bin);
  temporaryDirectories.push(directory);

  writeExecutable(
    path.join(bin, "git"),
    `#!/usr/bin/env bash
set -euo pipefail
{
  printf 'git'
  printf '\\t%s' "$@"
  printf '\\n'
} >> "\${COMMAND_LOG}"

if [[ "\${1:-}" == "diff" ]]; then
  exit "\${GIT_DIFF_EXIT}"
fi
if [[ "\${1:-}" == "ls-remote" && -n "\${REMOTE_SHA}" ]]; then
  printf '%s\\trefs/heads/chore/homebrew-test\\n' "\${REMOTE_SHA}"
fi
if [[ "\${1:-}" == "rev-parse" ]]; then
  printf '%s\\n' "\${HEAD_SHA}"
fi
`,
  );

  writeExecutable(
    path.join(bin, "gh"),
    `#!/usr/bin/env bash
set -euo pipefail
{
  printf 'gh'
  printf '\\t%s' "$@"
  printf '\\n'
} >> "\${COMMAND_LOG}"

if [[ "\${1:-} \${2:-}" == "pr list" ]]; then
  if [[ " $* " == *" --state open "* ]]; then
    printf '%s\\n' "\${OPEN_PR}"
  elif [[ " $* " == *" --state closed "* ]]; then
    printf '%s\\n' "\${CLOSED_PR}"
  fi
elif [[ "\${1:-} \${2:-}" == "pr create" ]]; then
  if [[ " $* " == *" --json "* ]]; then
    echo 'unknown flag: --json' >&2
    exit 1
  fi
  if [[ "\${CREATE_PR_FAILURE}" == "1" ]]; then
    echo 'GraphQL: GitHub Actions is not permitted to create or approve pull requests' >&2
    exit 1
  fi
  echo 'https://github.com/BrainerVirus/mochi/pull/42'
elif [[ "\${1:-} \${2:-}" == "pr view" ]]; then
  echo '42'
elif [[ "\${1:-} \${2:-}" == "pr checks" && " $* " == *" --json "* ]]; then
  echo '9'
elif [[ "\${1:-} \${2:-}" == "run list" ]]; then
  echo '9001'
fi
`,
  );

  return {
    commandLog,
    env: {
      ...process.env,
      CLOSED_PR: closedPr,
      COMMAND_LOG: commandLog,
      CREATE_PR_FAILURE: createPrFailure ? "1" : "0",
      GITHUB_RUN_ATTEMPT: "2",
      GITHUB_RUN_ID: "123",
      GITHUB_TOKEN: "test-token",
      GIT_DIFF_EXIT: hasChanges ? "1" : "0",
      HEAD_SHA: "abc123",
      OPEN_PR: openPr,
      PATH: `${bin}${path.delimiter}${process.env.PATH}`,
      REMOTE_SHA: remoteSha,
    },
  };
}

function publish(fixture) {
  return execFileSync(
    bashBin(),
    [
      script,
      "Casks/mochi-desktop.rb",
      "chore(homebrew): update test cask",
      "chore/homebrew-test",
      "chore(homebrew): update test cask",
    ],
    { encoding: "utf8", env: fixture.env, stdio: "pipe" },
  );
}

function commands(fixture) {
  return readFileSync(fixture.commandLog, "utf8").trim().split("\n");
}

afterEach(() => {
  for (const directory of temporaryDirectories.splice(0)) {
    rmSync(directory, { force: true, recursive: true });
  }
});

describe("publish-homebrew-cask-pr.sh", () => {
  it("passes bash -n", () => {
    expect(() => bashSyntaxCheck(script)).not.toThrow();
  });

  it("requires GITHUB_TOKEN and script arguments", () => {
    const bash = bashBin();
    expect(() => execFileSync(bash, [script], { encoding: "utf8", stdio: "pipe" })).toThrow();
  });

  it("creates, validates, and safely merges a cask PR", () => {
    const fixture = createFixture();

    expect(() => publish(fixture)).not.toThrow();

    const log = commands(fixture).join("\n");
    expect(log).toContain("gh\tpr\tcreate\t--base\tmain\t--head\tchore/homebrew-test");
    expect(log).not.toMatch(/gh\tpr\tcreate.*\t--json(?:\t|$)/);
    expect(log).toContain(
      "gh\tpr\tview\thttps://github.com/BrainerVirus/mochi/pull/42\t--json\tnumber\t--jq\t.number",
    );
    expect(log).toContain("gh\tpr\tchecks\t42\t--json\tname\t--jq\tlength");
    expect(log).toContain("gh\tpr\tchecks\t42\t--watch\t--interval\t15\t--fail-fast");
    expect(log).toContain(
      "gh\tpr\tmerge\t42\t--squash\t--delete-branch\t--match-head-commit\tabc123",
    );
  });

  it("updates a pre-existing branch and reuses its open PR", () => {
    const fixture = createFixture({ openPr: "17", remoteSha: "deadbeef" });

    expect(() => publish(fixture)).not.toThrow();

    const log = commands(fixture).join("\n");
    expect(log).toContain(
      "git\tpush\t--force-with-lease=refs/heads/chore/homebrew-test:deadbeef\t-u\torigin\tchore/homebrew-test",
    );
    expect(log).not.toContain("gh\tpr\tcreate");
    expect(log).toContain("gh\tpr\tmerge\t17");
  });

  it("explains the required Homebrew PR token permissions", () => {
    const fixture = createFixture({ createPrFailure: true });

    expect(() => publish(fixture)).toThrowError(
      /HOMEBREW_PR_TOKEN needs Actions read, Contents write, and Pull requests write/,
    );
  });

  it("reopens a closed unmerged PR instead of creating a duplicate", () => {
    const fixture = createFixture({ closedPr: "23", remoteSha: "deadbeef" });

    expect(() => publish(fixture)).not.toThrow();

    const log = commands(fixture).join("\n");
    expect(log).toContain("gh\tpr\treopen\t23");
    expect(log).not.toContain("gh\tpr\tcreate");
    expect(log).toContain("gh\tpr\tmerge\t23");
  });

  it("exits without GitHub calls when the cask is unchanged", () => {
    const fixture = createFixture({ hasChanges: false });

    expect(publish(fixture)).toContain("No cask changes to commit.");
    expect(commands(fixture)).not.toContain(expect.stringMatching(/^gh\t/));
  });
});
