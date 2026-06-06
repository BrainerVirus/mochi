import { z } from "zod";

import type { UpdateChannel } from "@/lib/schemas/settings";
import { appVersion } from "@/lib/tauri/commands";
import { cacheReleaseNotes, type ReleaseNotesCache } from "@/lib/updates/release-notes-cache";
import { sanitizeReleaseNotesForApp } from "@/lib/updates/sanitize-release-notes";

const GITHUB_RELEASE_API = "https://api.github.com/repos/BrainerVirus/mochi/releases/tags";

const GitHubReleaseResponseSchema = z.object({
  tag_name: z.string().optional(),
  body: z.string().nullable().optional(),
});

export function githubReleaseTagForChannel(channel: UpdateChannel, currentVersion: string): string {
  if (channel === "unstable") {
    return "unstable";
  }

  return currentVersion.startsWith("v") ? currentVersion : `v${currentVersion}`;
}

export function releaseNotesCacheKeyForChannel(channel: UpdateChannel): string {
  return `mochi:release-notes:${channel}:v1`;
}

export async function fetchCurrentReleaseNotes(
  channel: UpdateChannel,
): Promise<ReleaseNotesCache | null> {
  const version = await appVersion();
  const tag = githubReleaseTagForChannel(channel, version);
  const response = await fetch(`${GITHUB_RELEASE_API}/${encodeURIComponent(tag)}`);

  if (!response.ok) {
    return null;
  }

  const parsed = GitHubReleaseResponseSchema.safeParse(await response.json());
  if (!parsed.success) {
    return null;
  }

  const release = parsed.data;
  const notes = sanitizeReleaseNotesForApp(release.body);
  if (!notes) {
    return null;
  }

  const entry = {
    version: release.tag_name ?? tag,
    notes,
    channel,
    cachedAt: new Date().toISOString(),
    source: "installed-release" as const,
  };
  cacheReleaseNotes(entry);
  return entry;
}
