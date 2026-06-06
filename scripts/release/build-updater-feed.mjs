import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";

export const ENDPOINTS = [
  ["darwin", "aarch64"],
  ["darwin", "x86_64"],
  ["linux", "x86_64"],
  ["windows", "x86_64"],
];

export function endpointToPlatformKey(target, arch) {
  const key = `${target}-${arch}`;
  if (key === "darwin-aarch64") return key;
  if (key === "darwin-x86_64") return key;
  if (key === "linux-x86_64") return key;
  if (key === "windows-x86_64") return key;
  throw new Error(`unsupported updater endpoint: ${target}/${arch}`);
}

export function supportedRecoveryVersions(extraVersions = []) {
  return Array.from(new Set(["0.1.7", "0.2.0", ...extraVersions])).sort((a, b) =>
    a.localeCompare(b, undefined, { numeric: true }),
  );
}

export function buildUpdaterFeedEntries({
  versions,
  channels,
  latestVersion,
  notes,
  pubDate,
  artifacts,
}) {
  const entries = [];
  for (const version of versions) {
    for (const channel of channels) {
      for (const [target, arch] of ENDPOINTS) {
        const platformKey = endpointToPlatformKey(target, arch);
        const artifact = artifacts[platformKey];
        if (!artifact) {
          throw new Error(`missing updater artifact for ${platformKey}`);
        }
        entries.push({
          path: `updates/${target}/${arch}/${version}/${channel}.json`,
          json: {
            version: latestVersion,
            notes,
            pub_date: pubDate,
            platforms: {
              [platformKey]: artifact,
            },
          },
        });
      }
    }
  }
  return entries;
}

function parseArgs(argv) {
  return Object.fromEntries(
    argv.map((arg) => {
      const [key, ...valueParts] = arg.split("=");
      return [key.replace(/^--/, ""), valueParts.join("=")];
    }),
  );
}

export async function buildUpdaterFeedFromManifest({ manifestPath, outDir }) {
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  const versions = supportedRecoveryVersions(manifest.versions ?? [manifest.latestVersion]);
  const entries = buildUpdaterFeedEntries({
    versions,
    channels: manifest.channels,
    latestVersion: manifest.latestVersion,
    notes: manifest.notes,
    pubDate: manifest.pubDate,
    artifacts: manifest.artifacts,
  });

  for (const entry of entries) {
    const outputPath = join(outDir, entry.path);
    await mkdir(dirname(outputPath), { recursive: true });
    await writeFile(outputPath, `${JSON.stringify(entry.json, null, 2)}\n`);
  }

  return entries;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const args = parseArgs(process.argv.slice(2));
  if (!args.manifest || !args.out) {
    throw new Error("usage: build-updater-feed.mjs --manifest=updater-feed.json --out=public");
  }
  await buildUpdaterFeedFromManifest({
    manifestPath: args.manifest,
    outDir: args.out,
  });
}
