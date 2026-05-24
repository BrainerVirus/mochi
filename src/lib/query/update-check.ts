import { queryOptions } from "@tanstack/react-query";

import { queryKeys } from "@/lib/query/keys";
import { checkForUpdate } from "@/lib/tauri/commands";
import { cacheReleaseNotes } from "@/lib/updates/release-notes-cache";

export function createUpdateCheckQueryOptions(channel: string) {
  return queryOptions({
    queryKey: queryKeys.updateCheck(channel),
    queryFn: async () => {
      const info = await checkForUpdate(channel);
      if (info.notes && info.version) {
        cacheReleaseNotes({
          version: info.version,
          notes: info.notes,
          channel: info.channel,
          cachedAt: new Date().toISOString(),
        });
      }
      return info;
    },
    staleTime: 30 * 60_000,
    retry: 1,
  });
}
