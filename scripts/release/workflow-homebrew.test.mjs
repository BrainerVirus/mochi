import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

const prWorkflow = readFileSync(".github/workflows/pr.yml", "utf8");
const stableWorkflow = readFileSync(".github/workflows/release-stable.yml", "utf8");
const unstableWorkflow = readFileSync(".github/workflows/release-unstable.yml", "utf8");

describe("Homebrew release workflow contracts", () => {
  it("runs cask changes through normal pull request validation", () => {
    expect(prWorkflow).not.toContain("workflow_dispatch:");
    expect(prWorkflow).not.toMatch(/pull_request:[\s\S]*?paths-ignore:[\s\S]*?Casks\/\*\*/);
  });

  it("does not start an unstable release for cask-only main updates", () => {
    expect(unstableWorkflow).toMatch(/push:[\s\S]*?paths-ignore:[\s\S]*?Casks\/\*\*/);
  });

  it.each([
    ["stable", stableWorkflow],
    ["unstable", unstableWorkflow],
  ])("uses the dedicated PR token in the %s Homebrew job", (_channel, workflow) => {
    expect(workflow).toMatch(
      /update-homebrew-cask:[\s\S]*?token: \$\{\{ secrets\.HOMEBREW_PR_TOKEN \}\}/,
    );
    expect(workflow).toMatch(
      /update-homebrew-cask:[\s\S]*?GITHUB_TOKEN: \$\{\{ secrets\.HOMEBREW_PR_TOKEN \}\}/,
    );
  });
});
