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
    if (existsSync(child) && statSync(child).isDirectory()) {
      entries.push(...collectPaths(child, `${prefix}/${entry.name}`));
    }
  }
  return entries;
};

const ROOT_AREAS = [
  { dir: "lib", prefix: "@/lib" },
  { dir: "components", prefix: "@/components" },
  { dir: "hooks", prefix: "@/hooks" },
  { dir: "shared/components", prefix: "@/components" },
  { dir: "shared/hooks", prefix: "@/hooks" },
  { dir: "features/tray/components", prefix: "@/features/tray/components" },
  { dir: "features/tray/hooks", prefix: "@/features/tray/hooks" },
  { dir: "features/tray/lib", prefix: "@/features/tray/lib" },
];

const allEntries = {};
for (const { dir, prefix } of ROOT_AREAS) {
  const fullDir = path.join(srcRoot, dir);
  if (!existsSync(fullDir)) continue;
  const entries = collectPaths(fullDir, prefix);
  for (const { alias, target } of entries) {
    if (!allEntries[alias]) allEntries[alias] = [target];
  }
}

process.stdout.write(JSON.stringify(allEntries, null, 2));
