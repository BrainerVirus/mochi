import { z } from "zod";

import { sanitizeReleaseNotesForApp } from "@/lib/updates/sanitize-release-notes";

export const RELEASE_NOTES_CACHE_KEY = "mochi:release-notes:v1";
export const POST_UPDATE_REFRESH_KEY = "mochi:post-update-refresh:v1";

const ReleaseNotesCacheSchema = z.object({
  version: z.string(),
  notes: z.string(),
  channel: z.string(),
  cachedAt: z.string(),
  source: z.enum(["updater", "installed-release"]).default("updater"),
});

export type ReleaseNotesCache = z.infer<typeof ReleaseNotesCacheSchema>;
type ReleaseNotesCacheInput = z.input<typeof ReleaseNotesCacheSchema>;

export function cacheReleaseNotes(entry: ReleaseNotesCacheInput): void {
  try {
    const value = JSON.stringify(entry);
    localStorage.setItem(RELEASE_NOTES_CACHE_KEY, value);
    localStorage.setItem(`mochi:release-notes:${entry.channel}:v1`, value);
  } catch {
    // Private browsing or quota exceeded.
  }
}

export function readCachedReleaseNotes(): ReleaseNotesCache | null {
  try {
    const raw = localStorage.getItem(RELEASE_NOTES_CACHE_KEY);
    if (!raw) {
      return null;
    }

    const parsed = ReleaseNotesCacheSchema.safeParse(JSON.parse(raw));
    return parsed.success ? sanitizeCachedReleaseNotes(parsed.data) : null;
  } catch {
    return null;
  }
}

export function readCachedReleaseNotesForChannel(channel: string): ReleaseNotesCache | null {
  const channelKey = `mochi:release-notes:${channel}:v1`;
  try {
    const raw = localStorage.getItem(channelKey) ?? localStorage.getItem(RELEASE_NOTES_CACHE_KEY);
    if (!raw) {
      return null;
    }

    const parsed = ReleaseNotesCacheSchema.safeParse(JSON.parse(raw));
    return parsed.success ? sanitizeCachedReleaseNotes(parsed.data) : null;
  } catch {
    return null;
  }
}

function sanitizeCachedReleaseNotes(entry: ReleaseNotesCache): ReleaseNotesCache {
  return {
    ...entry,
    notes: sanitizeReleaseNotesForApp(entry.notes),
    source: entry.source ?? "updater",
  };
}

export function markPostUpdateRefreshPending(): void {
  try {
    sessionStorage.setItem(POST_UPDATE_REFRESH_KEY, "1");
  } catch {
    // Ignore storage failures.
  }
}

export function consumePostUpdateRefreshPending(): boolean {
  try {
    const pending = sessionStorage.getItem(POST_UPDATE_REFRESH_KEY) === "1";
    if (pending) {
      sessionStorage.removeItem(POST_UPDATE_REFRESH_KEY);
    }
    return pending;
  } catch {
    return false;
  }
}
