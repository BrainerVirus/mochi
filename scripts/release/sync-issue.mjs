#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const LABELS = ["release-automation", "automation"];
const LABEL_COLORS = { "release-automation": "d73a4a", automation: "0075ca" };

export function defaultRun(args) {
  return execFileSync("gh", args, { encoding: "utf8" });
}

export function renderSyncIssueTitle(tag) {
  return `chore(release): manifest sync failed for ${tag}`;
}

export function renderSyncIssueBody({ tag, error, runUrl }) {
  return [
    `Manifest sync or auto-merge failed for ${tag}.`,
    "",
    `Run: ${runUrl}`,
    `Tag: ${tag}`,
    "",
    "Error:",
    error,
    "",
    "See the job log for details.",
  ].join("\n");
}

export function openOrUpdateSyncIssue({ tag, error, runUrl, run = defaultRun }) {
  const original = error instanceof Error ? error : new Error(String(error));
  const title = renderSyncIssueTitle(tag);
  const body = renderSyncIssueBody({ tag, error: original.message, runUrl });
  try {
    for (const label of LABELS) {
      try {
        run(["label", "create", label, "--color", LABEL_COLORS[label]]);
      } catch {
        // non-fatal: label already exists or labels are unavailable
      }
    }
    const existing = JSON.parse(
      run([
        "issue",
        "list",
        "--label",
        "release-automation",
        "--state",
        "open",
        "--json",
        "number,title",
      ]),
    );
    if (existing.length === 0) {
      run([
        "issue",
        "create",
        "--title",
        title,
        "--body",
        body,
        "--label",
        "release-automation",
        "--label",
        "automation",
      ]);
    } else {
      run(["issue", "comment", String(existing[0].number), "--body", body]);
    }
  } catch (reportError) {
    console.error(
      `sync-issue report failed: ${reportError instanceof Error ? reportError.message : reportError}`,
    );
  }
  throw original;
}

function argValue(name) {
  const args = process.argv.slice(2);
  const idx = args.findIndex((a) => a === name || a.startsWith(`${name}=`));
  if (idx === -1) return undefined;
  if (args[idx].startsWith(`${name}=`)) return args[idx].slice(name.length + 1);
  return args[idx + 1];
}

if (resolve(process.argv[1] ?? "") === fileURLToPath(import.meta.url)) {
  const tag = argValue("--tag") ?? "unknown";
  const runUrl = argValue("--run-url") ?? "(unknown)";
  const error = argValue("--error") ?? "manifest sync or auto-merge failed (see job log)";
  try {
    openOrUpdateSyncIssue({ tag, error, runUrl });
  } catch (err) {
    console.error(err instanceof Error ? err.message : err);
    process.exit(1);
  }
}
