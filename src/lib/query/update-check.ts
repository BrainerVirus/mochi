import { queryOptions } from "@tanstack/react-query";

import { queryKeys } from "@/lib/query/keys";
import type { UpdateChannel } from "@/lib/schemas/settings";
import { checkForUpdate } from "@/lib/tauri/commands";
import { fetchCurrentReleaseNotes } from "@/lib/updates/current-release-notes";
import { cacheReleaseNotes } from "@/lib/updates/release-notes-cache";

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

      if (info.notes && info.version) {
        cacheReleaseNotes({
          version: info.version,
          notes: info.notes,
          channel: info.channel,
          cachedAt: new Date().toISOString(),
        });
      } else {
        await fetchCurrentReleaseNotes(channel).catch(() => null);
      }
      return info;
    },
    staleTime: 30 * 60_000,
    retry: 1,
  });
}
