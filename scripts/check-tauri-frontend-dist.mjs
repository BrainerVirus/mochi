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

console.log("Tauri frontend dist includes index.html SPA shell.");
