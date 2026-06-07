import fs from "node:fs";
import path from "node:path";
import * as ts from "typescript";

const EXTENSIONS = [".tsx", ".ts", ".jsx", ".js", ".mts", ".mjs"] as const;

const plugin: ts.server.PluginModule = {
  create({ languageServiceHost }) {
    const cwd = languageServiceHost.getCurrentDirectory();
    const srcRoot = path.resolve(cwd, "src");
    return {
      resolveModuleNames(moduleNames, _containingFile, ..._rest) {
        return moduleNames.map((name) => {
          if (!name.startsWith("@/")) return undefined;
          if (/\.[mc]?[jt]sx?$/.test(name)) return undefined;
          const abs = path.resolve(srcRoot, name.slice(2));
          for (const ext of EXTENSIONS) {
            const candidate = path.join(abs, `${path.basename(abs)}${ext}`);
            if (fs.existsSync(candidate)) {
              return {
                resolvedFileName: candidate,
                extension: ext.slice(1) as ts.Extension,
                isExternalLibraryImport: false,
              } as ts.ResolvedModuleFull;
            }
          }
          return undefined;
        });
      },
    };
  },
};

export = plugin;
