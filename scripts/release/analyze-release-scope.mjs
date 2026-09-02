#!/usr/bin/env node
// Path-gated release bump: prints major|minor|patch when conventional-commit
// product changes exist since <lastTag>; prints nothing to skip the release.
import { execFileSync } from "node:child_process";

export const PRODUCT_PATHS = [
  "app/",
  "src/",
  "src-tauri/",
  "scripts/install/",
  "Casks/",
  "packaging/",
];

const BUMPS = [
  ["major", /^(\w+)(\(.+\))?!:/],
  ["major", /^break(ing)?( change)?:/],
  ["minor", /^feat(\(.+\))?:/],
  ["patch", /^(fix|perf)(\(.+\))?:/],
];

export function bumpFromSubjects(subjects) {
  for (const [bump, re] of BUMPS) {
    if (subjects.some((s) => re.test(s))) return bump;
  }
  return null;
}

export function changedProductPaths(files, productPaths = PRODUCT_PATHS) {
  return productPaths.filter((prefix) => files.some((f) => f.startsWith(prefix)));
}

function git(...args) {
  return execFileSync("git", args, { encoding: "utf8" }).trim();
}

export function collect(lastTag) {
  const range = lastTag ? `${lastTag}..HEAD` : "HEAD";
  const subjects = git("log", range, "--no-merges", "--pretty=format:%s")
    .split("\n")
    .filter(Boolean);
  const files = git("log", range, "--no-merges", "--pretty=format:", "--name-only")
    .split("\n")
    .map((f) => f.trim())
    .filter(Boolean);
  return { subjects, files };
}

let lastTag = process.argv[2] || "";
if (!lastTag) {
  try {
    lastTag = git("describe", "--tags", "--abbrev=0", "--match=v*");
  } catch {
    // no v* tag exists yet; empty lastTag means "all commits since repo start"
    lastTag = "";
  }
}
const { subjects, files } = collect(lastTag);
const bump = bumpFromSubjects(subjects);
const productChanged = changedProductPaths(files).length > 0;
if (bump && productChanged) console.log(bump);
