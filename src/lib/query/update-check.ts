import { queryOptions } from "@tanstack/react-query";

import { queryKeys } from "@/lib/query/keys";
import type { UpdateChannel } from "@/lib/schemas/settings";
import { checkForUpdate } from "@/lib/tauri/commands";
import { fetchCurrentReleaseNotes } from "@/lib/updates/current-release-notes";
import { cacheReleaseNotes } from "@/lib/updates/release-notes-cache";
import { sanitizeReleaseNotesForApp } from "@/lib/updates/sanitize-release-notes";

export function createUpdateCheckQueryOptions(channel: UpdateChannel) {
  return queryOptions({
    queryKey: queryKeys.updateCheck(channel),
    queryFn: async () => {
      let info;
      try {
        info = await checkForUpdate(channel);
      } catch (error) {
        await fetchCurrentReleaseNotes(channel).catch(() => null);
        throw error;
      }

      const notes = sanitizeReleaseNotesForApp(info.notes);
      const sanitizedInfo = { ...info, notes: notes || null };
      if (sanitizedInfo.notes && sanitizedInfo.version) {
        cacheReleaseNotes({
          version: sanitizedInfo.version,
          notes: sanitizedInfo.notes,
          channel: sanitizedInfo.channel,
          cachedAt: new Date().toISOString(),
          source: "updater",
        });
      } else {
        await fetchCurrentReleaseNotes(channel).catch(() => null);
      }
      return sanitizedInfo;
    },
    staleTime: 30 * 60_000,
    retry: 1,
  });
}
