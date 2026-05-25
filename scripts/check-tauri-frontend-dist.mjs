import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const shellPath = join(process.cwd(), ".output/public/index.html");

if (!existsSync(shellPath)) {
  console.error("Missing .output/public/index.html — run `pnpm build` before this check.");
  process.exit(1);
}

const shell = readFileSync(shellPath, "utf8");
if (shell.length === 0 || !/<html[\s>]/i.test(shell)) {
  console.error("Invalid Tauri SPA shell at .output/public/index.html");
  process.exit(1);
}

// Reject the legacy hand-written shell (script-only body) that breaks hydration on Linux.
const bodyMatch = shell.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
const bodyInner = bodyMatch?.[1]?.trim() ?? "";
const scriptOnlyBody =
  bodyInner.length > 0 &&
  !/<div[\s>]/i.test(bodyInner) &&
  /<script\b/i.test(bodyInner);

if (scriptOnlyBody) {
  console.error(
    "index.html looks like the legacy manual Tauri shell (script-only body). It must include a mount root (e.g. div#root).",
  );
  process.exit(1);
}

if (!/<div[^>]+id=["']root["']/i.test(shell)) {
  console.error("index.html must include <div id=\"root\"> for client hydration.");
  process.exit(1);
}

if (!/<script\b/i.test(shell)) {
  console.error("index.html must include TanStack Start bootstrap scripts.");
  process.exit(1);
}

console.log("Tauri frontend dist includes index.html with a client mount root.");
