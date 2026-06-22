import { execFileSync } from "node:child_process";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { bashBin, bashSyntaxCheck } from "../install/test-helpers.mjs";

const script = path.join(import.meta.dirname, "publish-homebrew-cask-pr.sh");

describe("publish-homebrew-cask-pr.sh", () => {
  it("passes bash -n", () => {
    expect(() => bashSyntaxCheck(script)).not.toThrow();
  });

  it("requires GITHUB_TOKEN and script arguments", () => {
    const bash = bashBin();
    expect(() =>
      execFileSync(bash, [script], { encoding: "utf8", stdio: "pipe" }),
    ).toThrow();
  });
});
