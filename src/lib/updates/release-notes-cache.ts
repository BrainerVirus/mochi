import { z } from "zod";

export const RELEASE_NOTES_CACHE_KEY = "mochi:release-notes:v1";
export const POST_UPDATE_REFRESH_KEY = "mochi:post-update-refresh:v1";

const ReleaseNotesCacheSchema = z.object({
  version: z.string(),
  notes: z.string(),
  channel: z.string(),
  cachedAt: z.string(),
});

export type ReleaseNotesCache = z.infer<typeof ReleaseNotesCacheSchema>;

export function cacheReleaseNotes(entry: ReleaseNotesCache): void {
  try {
    localStorage.setItem(RELEASE_NOTES_CACHE_KEY, JSON.stringify(entry));
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
    return parsed.success ? parsed.data : null;
  } catch {
    return null;
  }
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
