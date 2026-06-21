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

function versionFromTag(tagName, unstableBaseVersion) {
  const stable = /^v(?<version>\d+\.\d+\.\d+)$/.exec(tagName);
  if (stable?.groups?.version) return stable.groups.version;
  const unstable = /^unstable-(?<version>\d{8}\.\d{6})$/.exec(tagName);
  if (unstable?.groups?.version) {
    if (!unstableBaseVersion || !/^\d+\.\d+\.\d+$/.test(unstableBaseVersion)) {
      throw new Error("unstable releases require --unstableBaseVersion=X.Y.Z");
    }
    return `${unstableBaseVersion}-unstable.${unstable.groups.version}`;
  }
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
  unstableBaseVersion,
  releaseBaseUrl,
  releaseNotesPath,
  outputPath,
  pubDate = new Date().toISOString(),
}) {
  if (channel !== "stable" && channel !== "unstable") {
    throw new Error(`unsupported updater channel: ${channel}`);
  }
  const latestVersion = versionFromTag(tagName, unstableBaseVersion);

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
    versions: ["0.1.7", "0.2.0", latestVersion],
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
    unstableBaseVersion: args.unstableBaseVersion,
    releaseBaseUrl: args.releaseBaseUrl,
    releaseNotesPath: args.releaseNotesPath,
    outputPath: args.outputPath,
    pubDate: args.pubDate,
  });
}
