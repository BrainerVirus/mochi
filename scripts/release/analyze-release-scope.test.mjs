import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  bumpFromSubjects,
  changedProductPaths,
  collect,
  PRODUCT_PATHS,
} from "./analyze-release-scope.mjs";

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
  it("matches BREAKING-CHANGE hyphen synonym", () => {
    expect(bumpFromSubjects(["BREAKING-CHANGE: drop api"])).toBe("major");
    expect(bumpFromSubjects(["breaking-change: drop api"])).toBe("major");
  });
  it("matches git-generated Revert subject case-insensitively", () => {
    expect(bumpFromSubjects(['Revert "fix: bad deploy"'])).toBe("patch");
    expect(bumpFromSubjects(["Revert: restore login"])).toBe("patch");
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

describe("collect excludes manifest-sync commits", () => {
  it("ignores sync commit subjects and files so the gate stays closed", () => {
    const dir = mkdtempSync(join(tmpdir(), "mochi-release-gate-"));
    const git = (...args) => execFileSync("git", args, { cwd: dir, encoding: "utf8" });
    git("init");
    git("config", "user.email", "test@example.com");
    git("config", "user.name", "test");
    writeFileSync(join(dir, "README.md"), "init\n");
    git("add", "README.md");
    git("commit", "-m", "chore: init");
    git("tag", "v0.0.0");
    mkdirSync(join(dir, "docs"), { recursive: true });
    writeFileSync(join(dir, "docs", "note.md"), "note\n");
    git("add", "docs/note.md");
    git("commit", "-m", "fix: typo in docs");
    mkdirSync(join(dir, "src-tauri"), { recursive: true });
    writeFileSync(join(dir, "src-tauri", "Cargo.toml"), "[package]\n");
    git("add", "src-tauri/Cargo.toml");
    git("commit", "-m", "chore(release): sync manifests to v9.9.9");
    const { subjects, files } = collect("v0.0.0", { cwd: dir });
    expect(subjects).not.toContain("chore(release): sync manifests to v9.9.9");
    expect(files).not.toContain("src-tauri/Cargo.toml");
    expect(bumpFromSubjects(subjects)).toBe("patch");
    expect(changedProductPaths(files)).toEqual([]);
  });
});
