import { describe, expect, it } from "vitest";
import { openOrUpdateSyncIssue } from "./sync-issue.mjs";

function makeRun({ listJson = "[]" } = {}) {
  const calls = [];
  const run = (args) => {
    calls.push(args);
    if (args[0] === "label") return "";
    if (args[0] === "issue" && args[1] === "list") return listJson;
    return "";
  };
  return { calls, run };
}

const base = { tag: "v1.2.3", error: "boom", runUrl: "https://example.test/run/1" };

describe("sync-issue", () => {
  it("creates an issue with title/body/labels when none is open", () => {
    const { calls, run } = makeRun();
    expect(() => openOrUpdateSyncIssue({ ...base, run })).toThrow("boom");
    const create = calls.find((c) => c[0] === "issue" && c[1] === "create");
    expect(create).toBeDefined();
    expect(create).toContain("chore(release): manifest sync failed for v1.2.3");
    expect(create).toContain("release-automation");
    expect(create).toContain("automation");
    expect(create.join("\n")).toContain("v1.2.3");
    expect(create.join("\n")).toContain("boom");
  });
  it("comments instead of creating when an issue is already open", () => {
    const { calls, run } = makeRun({
      listJson: '[{"number":42,"title":"chore(release): manifest sync failed for v1.2.3"}]',
    });
    expect(() => openOrUpdateSyncIssue({ ...base, run })).toThrow("boom");
    expect(calls.some((c) => c[0] === "issue" && c[1] === "create")).toBe(false);
    const comment = calls.find((c) => c[0] === "issue" && c[1] === "comment");
    expect(comment).toBeDefined();
    expect(comment).toContain("42");
  });
  it("includes the run URL in the issue body", () => {
    const { calls, run } = makeRun();
    expect(() => openOrUpdateSyncIssue({ ...base, run })).toThrow("boom");
    const create = calls.find((c) => c[0] === "issue" && c[1] === "create");
    expect(create.join("\n")).toContain("https://example.test/run/1");
  });
  it("surfaces the original error when gh list fails", () => {
    const run = (args) => {
      if (args[0] === "issue" && args[1] === "list") throw new Error("gh exploded");
      return "";
    };
    expect(() => openOrUpdateSyncIssue({ ...base, run })).toThrow("boom");
    expect(() => openOrUpdateSyncIssue({ ...base, run })).not.toThrow("gh exploded");
  });
});
