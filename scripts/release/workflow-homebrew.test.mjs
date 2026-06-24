import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

const prWorkflow = readFileSync(".github/workflows/pr.yml", "utf8");
const stableWorkflow = readFileSync(".github/workflows/release-stable.yml", "utf8");
const unstableWorkflow = readFileSync(".github/workflows/release-unstable.yml", "utf8");

describe("Homebrew release workflow contracts", () => {
  it("supports explicit PR validation without approval-gated duplicate runs", () => {
    expect(prWorkflow).toContain("workflow_dispatch:");
    expect(prWorkflow).toContain("validation_id:");
    expect(prWorkflow).toMatch(/pull_request:[\s\S]*?paths-ignore:[\s\S]*?Casks\/\*\*/);
  });

  it("does not start an unstable release for cask-only main updates", () => {
    expect(unstableWorkflow).toMatch(/push:[\s\S]*?paths-ignore:[\s\S]*?Casks\/\*\*/);
  });

  it.each([
    ["stable", stableWorkflow],
    ["unstable", unstableWorkflow],
  ])("grants the %s Homebrew job permission to dispatch validation", (_channel, workflow) => {
    expect(workflow).toMatch(/update-homebrew-cask:[\s\S]*?permissions:[\s\S]*?actions: write/);
    expect(workflow).toMatch(/update-homebrew-cask:[\s\S]*?permissions:[\s\S]*?checks: read/);
  });
});
