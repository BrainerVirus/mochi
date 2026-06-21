import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";

const DEFAULT_BASE_URL = "https://brainervirus.github.io/mochi";
const DEFAULT_ATTEMPTS = 6;
const DEFAULT_DELAY_SECONDS = 10;

function parseArgs(argv) {
  const args = {
    baseUrl: DEFAULT_BASE_URL,
    attempts: DEFAULT_ATTEMPTS,
    delaySeconds: DEFAULT_DELAY_SECONDS,
    paths: [],
  };

  for (const arg of argv) {
    if (arg.startsWith("--baseUrl=")) {
      args.baseUrl = arg.slice("--baseUrl=".length).replace(/\/$/, "");
      continue;
    }
    if (arg.startsWith("--attempts=")) {
      args.attempts = Number.parseInt(arg.slice("--attempts=".length), 10);
      continue;
    }
    if (arg.startsWith("--delaySeconds=")) {
      args.delaySeconds = Number.parseInt(arg.slice("--delaySeconds=".length), 10);
      continue;
    }
    args.paths.push(arg);
  }

  if (args.paths.length === 0) {
    throw new Error(
      "usage: validate-published-feeds.mjs [--baseUrl=URL] [--attempts=N] [--delaySeconds=N] <feed-path> [...]",
    );
  }
  if (!Number.isFinite(args.attempts) || args.attempts < 1) {
    throw new Error("--attempts must be a positive integer");
  }
  if (!Number.isFinite(args.delaySeconds) || args.delaySeconds < 1) {
    throw new Error("--delaySeconds must be a positive integer");
  }

  return args;
}

function sleep(seconds) {
  spawnSync("sleep", [String(seconds)], { stdio: "inherit" });
}

function curlFeed(url, outputPath) {
  const result = spawnSync(
    "curl",
    ["--fail", "--silent", "--show-error", "--location", url, "--output", outputPath],
    { encoding: "utf8" },
  );
  if (result.status !== 0) {
    const detail = result.stderr?.trim() || result.stdout?.trim() || `exit ${result.status}`;
    throw new Error(detail);
  }
}

function validateFeed(outputPath) {
  const result = spawnSync("node", ["scripts/release/validate-updater-feed.mjs", outputPath], {
    encoding: "utf8",
    stdio: "pipe",
  });
  if (result.status !== 0) {
    const detail = result.stderr?.trim() || result.stdout?.trim() || `exit ${result.status}`;
    throw new Error(detail);
  }
}

export async function validatePublishedFeeds({
  baseUrl,
  paths,
  attempts = DEFAULT_ATTEMPTS,
  delaySeconds = DEFAULT_DELAY_SECONDS,
}) {
  if (!paths?.length) {
    throw new Error("validatePublishedFeeds requires at least one feed path");
  }
  for (const path of paths) {
    const url = `${baseUrl}/${path.replace(/^\//, "")}`;
    let lastError = null;

    for (let attempt = 1; attempt <= attempts; attempt += 1) {
      try {
        curlFeed(url, "feed.json");
        validateFeed("feed.json");
        lastError = null;
        break;
      } catch (error) {
        lastError = error;
        if (attempt < attempts) {
          console.log(
            `validate-published-feeds: ${path} not ready (attempt ${attempt}/${attempts}): ${error.message}`,
          );
          sleep(delaySeconds);
        }
      }
    }

    if (lastError) {
      throw new Error(`published feed failed for ${path}: ${lastError.message}`);
    }
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const args = parseArgs(process.argv.slice(2));
  await validatePublishedFeeds(args);
  writeFileSync("feed.json", "");
}
