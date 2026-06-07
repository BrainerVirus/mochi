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
      if (!source.startsWith(alias)) return null;
      if (/\.[mc]?[jt]sx?$/.test(source)) return null;
      const abs = path.resolve(srcRoot, source.slice(alias.length));
      for (const ext of EXTENSIONS) {
        const candidate = path.join(abs, `${path.basename(abs)}${ext}`);
        if (fs.existsSync(candidate)) {
          return candidate;
        }
      }
      return null;
    },
  };
};
