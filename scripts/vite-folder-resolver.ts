import fs from "node:fs";
import path from "node:path";
import type { Plugin } from "vite";

const EXTENSIONS = [".tsx", ".ts", ".jsx", ".js", ".mts", ".mjs"] as const;

export interface FolderResolverOptions {
  srcRoot: string;
  alias?: string;
}

export const folderResolver = ({ srcRoot, alias = "@/" }: FolderResolverOptions): Plugin => {
  return {
    name: "mochi:folder-resolver",
    enforce: "pre",
    async resolveId(source, _importer) {
      // Pre-alias: `@/foo/bar` is the form used in source code.
      if (source.startsWith(alias)) {
        if (/\.[mc]?[jt]sx?$/.test(source)) return null;
        const abs = path.resolve(srcRoot, source.slice(alias.length));
        for (const ext of EXTENSIONS) {
          const candidate = path.join(abs, `${path.basename(abs)}${ext}`);
          if (fs.existsSync(candidate)) {
            return candidate;
          }
        }
        return null;
      }
      // Post-alias: Vite's resolve.alias has already turned `@/foo/bar` into
      // `<srcRoot>/foo/bar`. If that path is a directory, look for a same-named
      // file inside.
      if (path.isAbsolute(source) && source.startsWith(srcRoot)) {
        if (/\.[mc]?[jt]sx?$/.test(source)) return null;
        if (fs.existsSync(source) && fs.statSync(source).isDirectory()) {
          for (const ext of EXTENSIONS) {
            const candidate = path.join(source, `${path.basename(source)}${ext}`);
            if (fs.existsSync(candidate)) {
              return candidate;
            }
          }
        }
      }
      return null;
    },
  };
};
