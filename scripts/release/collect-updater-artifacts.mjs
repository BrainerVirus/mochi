import { execFileSync } from "node:child_process";
import { readdir, readFile, writeFile } from "node:fs/promises";
import { basename, join, sep } from "node:path";

const REQUIRED_ARTIFACTS = {
  "darwin-aarch64": [/aarch64.*macos.*Mochi\.app\.tar\.gz$/i, /Mochi_aarch64\.app\.tar\.gz$/i],
  "darwin-x86_64": [/x86_64.*macos.*Mochi\.app\.tar\.gz$/i, /Mochi_x64\.app\.tar\.gz$/i],
  "linux-x86_64": [/appimage.*amd64\.AppImage$/i, /Mochi_.*amd64\.AppImage$/i],
  "windows-x86_64": [/nsis.*x64-setup\.exe$/i, /Mochi_.*x64-setup\.exe$/i],
};

async function listFiles(root) {
  const entries = await readdir(root, { recursive: true, withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile())
    .map((entry) => {
      const parentPath = entry.parentPath ?? root;
      return join(parentPath, entry.name);
    });
}

function toPosixPath(filePath) {
  return filePath.split(sep).join("/");
}

const RECOVERY_VERSIONS = ["0.1.7", "0.2.0"];

function stableVersionFromTag(tagName) {
  return /^v(?<version>\d+\.\d+\.\d+)$/.exec(tagName)?.groups?.version;
}

function listReleaseTags() {
  const output = execFileSync("git", ["tag", "--list", "v*"], { encoding: "utf8" });
  return output
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

export function resolveFeedVersions({ latestVersion, tags } = {}) {
  let discovered = [];
  try {
    const names = tags ?? listReleaseTags();
    discovered = names.map((tag) => stableVersionFromTag(tag)).filter(Boolean);
  } catch {
    discovered = [];
  }
  const candidates = latestVersion
    ? [...RECOVERY_VERSIONS, ...discovered, latestVersion]
    : [...RECOVERY_VERSIONS, ...discovered];
  return [...new Set(candidates)].sort((a, b) => a.localeCompare(b, undefined, { numeric: true }));
}

function versionFromTag(tagName) {
  const stable = /^v(?<version>\d+\.\d+\.\d+)$/.exec(tagName);
  if (stable?.groups?.version) return stable.groups.version;
  throw new Error(`unsupported release tag for updater feed: ${tagName}`);
}

async function notesFromPath(path) {
  try {
    return await readFile(path, "utf8");
  } catch {
    return "### What's changed\n- See the GitHub release notes for this version.";
  }
}

export async function collectUpdaterArtifacts({
  artifactRoot,
  channel,
  tagName,
  releaseBaseUrl,
  releaseNotesPath,
  outputPath,
  tags,
  pubDate = new Date().toISOString(),
}) {
  if (channel !== "stable") {
    throw new Error(`unsupported updater channel: ${channel}`);
  }
  const latestVersion = versionFromTag(tagName);

  const files = await listFiles(artifactRoot);
  const artifacts = {};
  for (const [platform, patterns] of Object.entries(REQUIRED_ARTIFACTS)) {
    const artifactPath = files.find(
      (file) =>
        !file.endsWith(".sig") && patterns.some((pattern) => pattern.test(toPosixPath(file))),
    );
    if (!artifactPath) throw new Error(`missing updater artifact for ${platform}`);

    const signaturePath = `${artifactPath}.sig`;
    if (!files.includes(signaturePath)) {
      throw new Error(`missing updater signature for ${platform}: ${signaturePath}`);
    }

    artifacts[platform] = {
      url: `${releaseBaseUrl}/${basename(artifactPath)}`,
      signature: (await readFile(signaturePath, "utf8")).trim(),
    };
  }

  const manifest = {
    latestVersion,
    channels: [channel],
    versions: resolveFeedVersions({ latestVersion, tags }),
    notes: await notesFromPath(releaseNotesPath),
    pubDate,
    artifacts,
  };

  await writeFile(outputPath, `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

function parseArgs(argv) {
  return Object.fromEntries(
    argv.map((arg) => {
      const [key, ...valueParts] = arg.split("=");
      return [key.replace(/^--/, ""), valueParts.join("=")];
    }),
  );
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const args = parseArgs(process.argv.slice(2));
  await collectUpdaterArtifacts({
    artifactRoot: args.artifactRoot,
    channel: args.channel,
    tagName: args.tagName,
    releaseBaseUrl: args.releaseBaseUrl,
    releaseNotesPath: args.releaseNotesPath,
    outputPath: args.outputPath,
    pubDate: args.pubDate,
  });
}
