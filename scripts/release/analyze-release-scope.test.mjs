import { spawnSync } from "node:child_process";
import { describe, expect, it } from "vitest";
import { bumpFromSubjects, changedProductPaths, PRODUCT_PATHS } from "./analyze-release-scope.mjs";

describe("bumpFromSubjects", () => {
  it("returns major for breaking change", () => {
    expect(bumpFromSubjects(["feat!: x", "fix: y"])).toBe("major");
  });
  it("returns major for uppercase BREAKING CHANGE subject", () => {
    expect(bumpFromSubjects(["BREAKING CHANGE: drop api"])).toBe("major");
  });
  it("returns patch for revert subjects (semantic-release convention)", () => {
    expect(bumpFromSubjects(["revert: restore login"])).toBe("patch");
  });
  it("matches lowercase breaking/break forms", () => {
    expect(bumpFromSubjects(["breaking: drop"])).toBe("major");
    expect(bumpFromSubjects(["break: drop"])).toBe("major");
    expect(bumpFromSubjects(["breaking change: drop"])).toBe("major");
  });
  it("keeps chore!/docs! as major when product paths change (safe gate direction: under-release is worse than over-release)", () => {
    expect(bumpFromSubjects(["chore!: rotate keys"])).toBe("major");
    expect(bumpFromSubjects(["docs!: rewrite api"])).toBe("major");
  });
  it("returns feat over fix", () => {
    expect(bumpFromSubjects(["fix: y", "feat: x"])).toBe("minor");
  });
  it("returns patch for fix/perf only", () => {
    expect(bumpFromSubjects(["fix: y", "perf: z"])).toBe("patch");
  });
  it("returns null for docs/chore/ci only", () => {
    expect(bumpFromSubjects(["docs: x", "chore: y", "ci: z"])).toBeNull();
  });
});

describe("changedProductPaths", () => {
  it("detects product path changes", () => {
    const files = ["docs/tech-stack.md", "src-tauri/src/lib.rs", "README.md"];
    expect(changedProductPaths(files, PRODUCT_PATHS)).toEqual(["src-tauri/"]);
  });
  it("returns empty for docs-only change", () => {
    expect(changedProductPaths(["docs/tech-stack.md"], PRODUCT_PATHS)).toEqual([]);
  });
  it("matches nested app routes", () => {
    expect(changedProductPaths(["app/routes/index.tsx"], PRODUCT_PATHS)).toEqual(["app/"]);
  });
});

describe("invalid tag argument", () => {
  it("fails gracefully with exit 0 and no stdout (gate closed, no stack)", () => {
    const result = spawnSync(
      process.execPath,
      ["scripts/release/analyze-release-scope.mjs", "this-tag-does-not-exist-12345"],
      { encoding: "utf8" },
    );
    expect(result.status).toBe(0);
    expect(result.stdout.trim()).toBe("");
    expect(result.stderr).toMatch(/cannot resolve tag|skipping release/i);
  });
});
