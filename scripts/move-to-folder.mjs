#!/usr/bin/env node
// Move a flat source file (e.g. src/lib/utils/foo.ts) into a per-unit folder
// (src/lib/utils/foo/foo.ts) so the folder-resolver plugin can map
// `@/lib/utils/foo` -> `src/lib/utils/foo/foo.ts`.
//
// If a colocated `foo.test.ts` (or `.tsx`) exists beside the source file,
// it moves with the source into the same folder.
//
// Relative imports inside the moved file are rewritten: any `./bar` or
// `../baz` that points outside the new per-unit folder becomes a `@/` alias,
// so callers don't need to be touched.
//
// Usage: node scripts/move-to-folder.mjs <path-to-file>
import { execSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const file = process.argv[2];
if (!file) {
  console.error("Usage: node scripts/move-to-folder.mjs <path-to-file>");
  process.exit(1);
}

const abs = path.resolve(file);
const dir = path.dirname(abs);
const base = path.basename(abs);
const stem = base.replace(/\.(test\.)?tsx?$/, "");
const isTest = /\.test\.[mc]?[jt]sx?$/.test(base);
const targetDir = path.join(dir, stem);

if (existsSync(targetDir)) {
  console.error(`Target folder already exists: ${targetDir}`);
  process.exit(1);
}

execSync(`mkdir -p "${targetDir}"`);
execSync(`git mv "${abs}" "${path.join(targetDir, base)}"`);

if (!isTest) {
  for (const ext of [".ts", ".tsx", ".mts", ".mjs"]) {
    const sibling = path.join(dir, `${stem}.test${ext}`);
    if (existsSync(sibling)) {
      execSync(`git mv "${sibling}" "${path.join(targetDir, `${stem}.test${ext}`)}"`);
      break;
    }
  }
}

const newBase = path.join(targetDir, base);
const newBaseContent = readFileSync(newBase, "utf8");
const srcRoot = path.resolve(process.cwd(), "src");

const rewrite = (content) => {
  return content.replace(/from\s+["'](\.\.?\/[^"']+)["']/g, (match, relPath) => {
    const newResolved = path.resolve(targetDir, relPath);
    if (existsSync(`${newResolved}.ts`) || existsSync(`${newResolved}.tsx`)) {
      return match;
    }
    const oldResolved = path.resolve(dir, relPath);
    const aliasPath = "/" + path.relative(srcRoot, oldResolved).split(path.sep).join("/");
    return `from "@${aliasPath}"`;
  });
};

writeFileSync(newBase, rewrite(newBaseContent));

const newTestPath = path.join(targetDir, `${stem}.test.ts`);
if (existsSync(newTestPath)) {
  writeFileSync(newTestPath, rewrite(readFileSync(newTestPath, "utf8")));
}

console.log(`Moved ${file} -> ${path.relative(process.cwd(), newBase)}`);
