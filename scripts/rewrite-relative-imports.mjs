#!/usr/bin/env node
// Post-migration cleanup: rewrite any relative imports in moved files that
// point outside their per-unit folder to use `@/` aliases.
//
// Usage: node scripts/rewrite-relative-imports.mjs
import { readdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import path from "node:path";

const srcRoot = path.resolve(process.cwd(), "src");

const rewriteInDir = (dir) => {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      rewriteInDir(full);
    } else if (/\.[mc]?[jt]sx?$/.test(entry.name) && !entry.name.endsWith(".d.ts")) {
      const content = readFileSync(full, "utf8");
      const rewritten = content.replace(
        /from\s+["'](\.\.?\/[^"']+)["']/g,
        (match, relPath) => {
          const newResolved = path.resolve(dir, relPath);
          if (existsSync(`${newResolved}.ts`) || existsSync(`${newResolved}.tsx`)) {
            return match;
          }
          const oldResolved = path.resolve(path.dirname(dir), relPath);
          if (path.relative(srcRoot, oldResolved).startsWith("..")) return match;
          const aliasPath =
            "/" + path.relative(srcRoot, oldResolved).split(path.sep).join("/");
          return `from "@${aliasPath}"`;
        },
      );
      if (rewritten !== content) {
        writeFileSync(full, rewritten);
        console.log(`Rewrote ${path.relative(process.cwd(), full)}`);
      }
    }
  }
};

rewriteInDir(srcRoot);
