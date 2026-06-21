import { execFileSync } from "node:child_process";
import { describe, expect, it } from "vitest";

const repoRoot = new URL("../..", import.meta.url).pathname;

describe("setup-macos-brew-tap.sh", () => {
  it("parses under macOS /bin/bash 3.2", () => {
    expect(() =>
      execFileSync("/bin/bash", ["-n", "scripts/install/setup-macos-brew-tap.sh"], {
        cwd: repoRoot,
        encoding: "utf8",
      }),
    ).not.toThrow();
  });
});

describe("install-macos-brew.sh", () => {
  it("parses under macOS /bin/bash 3.2", () => {
    expect(() =>
      execFileSync("/bin/bash", ["-n", "scripts/install/install-macos-brew.sh"], {
        cwd: repoRoot,
        encoding: "utf8",
      }),
    ).not.toThrow();
  });
});
