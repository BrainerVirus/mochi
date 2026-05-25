import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
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

function isLegacyScriptOnlyShell(html: string): boolean {
  const bodyMatch = html.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
  const bodyInner = bodyMatch?.[1]?.trim() ?? "";
  return bodyInner.length > 0 && !/<div[\s>]/i.test(bodyInner) && /<script\b/i.test(bodyInner);
}

function hasRootAbsoluteBundledAssets(html: string): boolean {
  return /\b(?:src|href)=["']\/assets\//i.test(html);
}

/**
 * Writes `index.html` for Tauri after the client build when TanStack SPA prerender
 * cannot run (e.g. Nitro preview self-fetch failures). Includes a mount root and the
 * client entry script so the WebView can hydrate the router.
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
          const existing = readFileSync(SHELL_PATH, "utf8");
          if (!isLegacyScriptOnlyShell(existing) && !hasRootAbsoluteBundledAssets(existing)) {
            return;
          }
        }

        const entryJs = findAsset("index-", ".js");
        if (!entryJs) {
          throw new Error(
            "Missing client entry chunk in .output/public/assets — cannot write Tauri SPA shell.",
          );
        }

        const entryCss = findAsset("index-", ".css");
        const cssLink = entryCss
          ? `    <link rel="stylesheet" href="./assets/${entryCss}" />\n`
          : "";

        const html = `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Mochi</title>
${cssLink}  </head>
  <body>
    <div id="root"></div>
    <script type="module" crossorigin src="./assets/${entryJs}"></script>
  </body>
</html>
`;

        writeFileSync(SHELL_PATH, html);
      },
    },
  };
}
