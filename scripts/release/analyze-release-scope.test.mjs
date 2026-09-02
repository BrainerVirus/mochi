import { describe, expect, it } from "vitest";
import { bumpFromSubjects, changedProductPaths, PRODUCT_PATHS } from "./analyze-release-scope.mjs";

describe("bumpFromSubjects", () => {
  it("returns major for breaking change", () => {
    expect(bumpFromSubjects(["feat!: x", "fix: y"])).toBe("major");
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
