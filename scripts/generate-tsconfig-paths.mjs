#!/usr/bin/env node
// Generate explicit tsconfig `paths` entries for every per-unit folder
// under src/lib/, src/components/, src/hooks/. The entries map
// `@/foo/bar` to `./src/foo/bar/bar.{ts,tsx}` so that tools that don't
// run the folder-resolver plugin (oxlint type-aware, plain
// `tsc --noEmit`) still find the file.
//
// Usage: node scripts/generate-tsconfig-paths.mjs
import { readdirSync, statSync, existsSync } from "node:fs";
import path from "node:path";

const srcRoot = path.resolve(process.cwd(), "src");
const EXTENSIONS = [".tsx", ".ts", ".jsx", ".js", ".mts", ".mjs"];

const findSameNamedFile = (dir) => {
  const base = path.basename(dir);
  for (const ext of EXTENSIONS) {
    const candidate = path.join(dir, `${base}${ext}`);
    if (existsSync(candidate)) return candidate;
  }
  return null;
};

const collectPaths = (dir, prefix) => {
  const entries = [];
  const selfFile = findSameNamedFile(dir);
  if (selfFile) {
    const alias = prefix;
    const target = "./" + path.relative(process.cwd(), selfFile).split(path.sep).join("/");
    entries.push({ alias, target });
  }
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const child = path.join(dir, entry.name);
    if (existsSync(child) && statSync(child).isDirectory() && findSameNamedFile(child)) {
      entries.push(...collectPaths(child, `${prefix}/${entry.name}`));
    }
  }
  return entries;
};

const ROOT_AREAS = ["lib", "components", "hooks"];

const allEntries = {};
for (const area of ROOT_AREAS) {
  const dir = path.join(srcRoot, area);
  if (!existsSync(dir)) continue;
  const entries = collectPaths(dir, `@/${area}`);
  for (const { alias, target } of entries) {
    allEntries[alias] = [target];
  }
}

process.stdout.write(JSON.stringify(allEntries, null, 2));
