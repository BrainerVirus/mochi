import { readFile } from "node:fs/promises";

export async function validateFeedFile(path) {
  const parsed = JSON.parse(await readFile(path, "utf8"));
  if (!parsed.version || !parsed.pub_date || !parsed.platforms) {
    throw new Error(`invalid updater feed: ${path}`);
  }
  for (const [platform, artifact] of Object.entries(parsed.platforms)) {
    if (!artifact?.url || !artifact?.signature) {
      throw new Error(`invalid updater artifact for ${platform} in ${path}`);
    }
  }
  return true;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const paths = process.argv.slice(2);
  if (paths.length === 0) {
    throw new Error("usage: validate-updater-feed.mjs <feed.json> [...]");
  }
  for (const path of paths) {
    await validateFeedFile(path);
  }
}
