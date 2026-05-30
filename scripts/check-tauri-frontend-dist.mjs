import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const DEFAULT_PUBLIC_DIR = join(process.cwd(), "dist");

function isRelativeBundledAssetPath(path) {
  return (
    path.startsWith("./assets/") || path.startsWith("assets/") || path.startsWith("/./assets/")
  );
}

function collectBundledAssetRefs(shell) {
  return Array.from(shell.matchAll(/\b(?:src|href)=["']([^"']*assets\/[^"']+)["']/gi)).map(
    (match) => match[1],
  );
}

function hasClientHydrationTarget(shell) {
  return /<div[^>]+id=["']root["']/i.test(shell);
}

function jsChunksConstructRootAbsoluteAssetUrls(publicDir) {
  const assetsDir = join(publicDir, "assets");
  if (!existsSync(assetsDir)) {
    return false;
  }

  return readdirSync(assetsDir)
    .filter((file) => file.endsWith(".js"))
    .some((file) => {
      const chunk = readFileSync(join(assetsDir, file), "utf8");
      return /["'`]\/assets\//.test(chunk) || /(?:return|=>)\s*["'`]\/["'`]\s*\+/.test(chunk);
    });
}

export function validateTauriFrontendDist(publicDir = DEFAULT_PUBLIC_DIR) {
  const errors = [];
  const shellPath = join(publicDir, "index.html");

  if (!existsSync(shellPath)) {
    return ["Missing dist/index.html — run `pnpm build` before this check."];
  }

  const shell = readFileSync(shellPath, "utf8");
  if (shell.length === 0 || !/<html[\s>]/i.test(shell)) {
    errors.push("Invalid Tauri frontend document at dist/index.html");
  }

  // Reject the legacy hand-written shell (script-only body) that breaks hydration on Linux.
  const bodyMatch = shell.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
  const bodyInner = bodyMatch?.[1]?.trim() ?? "";
  const scriptOnlyBody =
    bodyInner.length > 0 &&
    !/<div[\s>]/i.test(bodyInner) &&
    !hasClientHydrationTarget(shell) &&
    /<script\b/i.test(bodyInner);

  if (scriptOnlyBody) {
    errors.push(
      "index.html looks like the legacy manual Tauri shell (script-only body). It must include a mount root (e.g. div#root).",
    );
  }

  if (!hasClientHydrationTarget(shell)) {
    errors.push("index.html must include the React mount target div#root.");
  }

  if (!/<script\b/i.test(shell)) {
    errors.push("index.html must include the Vite React bootstrap script.");
  }

  const assetRefs = collectBundledAssetRefs(shell);
  if (assetRefs.some((ref) => !isRelativeBundledAssetPath(ref))) {
    errors.push("index.html must use relative asset URLs for Tauri packaged webviews.");
  }

  for (const ref of assetRefs.filter(isRelativeBundledAssetPath)) {
    const normalizedRef = ref.startsWith("./")
      ? ref.slice(2)
      : ref.startsWith("/./")
        ? ref.slice(3)
        : ref;
    if (!existsSync(join(publicDir, normalizedRef))) {
      errors.push(`index.html references missing asset: ${ref}`);
    }
  }

  if (jsChunksConstructRootAbsoluteAssetUrls(publicDir)) {
    errors.push(
      "client chunks must not construct root-absolute /assets URLs for packaged Tauri webviews.",
    );
  }

  return errors;
}

export function runTauriFrontendDistCheck(publicDir = DEFAULT_PUBLIC_DIR) {
  const errors = validateTauriFrontendDist(publicDir);
  if (errors.length > 0) {
    for (const error of errors) {
      console.error(error);
    }
    process.exit(1);
  }

  console.log("Tauri frontend dist includes index.html with a React mount target.");
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  runTauriFrontendDistCheck();
}
