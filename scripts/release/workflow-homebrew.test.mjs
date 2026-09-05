import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

const prWorkflow = readFileSync(".github/workflows/pr.yml", "utf8");
const stableWorkflow = readFileSync(".github/workflows/release-stable.yml", "utf8");

describe("Homebrew release workflow contracts", () => {
  it("runs cask changes through normal pull request validation", () => {
    expect(prWorkflow).not.toContain("workflow_dispatch:");
    expect(prWorkflow).not.toMatch(/pull_request:[\s\S]*?paths-ignore:[\s\S]*?Casks\/\*\*/);
  });

  it.each([["stable", stableWorkflow]])(
    "uses GITHUB_TOKEN with write permissions in the %s Homebrew job",
    (_channel, workflow) => {
      const job = workflow.split("update-homebrew-cask:")[1];
      expect(job).toContain("contents: write");
      expect(job).toContain("pull-requests: write");
      expect(job).not.toContain("HOMEBREW_PR_TOKEN");
      expect(job).toMatch(/GITHUB_TOKEN: \$\{\{ secrets\.GITHUB_TOKEN \}\}/);
    },
  );
});
