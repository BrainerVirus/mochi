import { existsSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import type { Plugin } from "vite";

const PUBLIC_DIR = join(process.cwd(), ".output/public");
const SHELL_PATH = join(PUBLIC_DIR, "index.html");

function findAsset(prefix: string, extension: string): string | undefined {
  const assetsDir = join(PUBLIC_DIR, "assets");
  if (!existsSync(assetsDir)) {
    return undefined;
  }

  return readdirSync(assetsDir).find((file) => file.startsWith(prefix) && file.endsWith(extension));
}

/**
 * Writes `index.html` for Tauri `frontendDist` after the client build.
 * TanStack Start SPA prerender is not used here — the desktop bundle only needs a static shell.
 */
export function writeTauriSpaShell(): Plugin {
  return {
    name: "mochi:write-tauri-shell",
    apply: "build",
    closeBundle: {
      order: "post",
      sequential: true,
      handler() {
        if (existsSync(SHELL_PATH)) {
          return;
        }

        const entryJs = findAsset("index-", ".js");
        if (!entryJs) {
          throw new Error(
            "Missing client entry chunk in .output/public/assets — cannot write Tauri SPA shell.",
          );
        }

        const entryCss = findAsset("index-", ".css");
        const cssLink = entryCss
          ? `    <link rel="stylesheet" href="/assets/${entryCss}" />\n`
          : "";

        const html = `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Mochi</title>
${cssLink}  </head>
  <body>
    <script type="module" src="/assets/${entryJs}"></script>
  </body>
</html>
`;

        writeFileSync(SHELL_PATH, html);
      },
    },
  };
}
