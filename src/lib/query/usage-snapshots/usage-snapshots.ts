import { queryOptions } from "@tanstack/react-query";

import { getUsageStates } from "@/lib/tauri/commands";

import { queryKeys } from "@/lib/query/keys";
import { usageRefreshIntervalMs } from "@/lib/query/usage-refetch-interval";

export function createUsageSnapshotsQueryOptions(refreshIntervalSeconds?: number) {
  return queryOptions({
    queryKey: queryKeys.usageSnapshots,
    queryFn: getUsageStates,
    ...(refreshIntervalSeconds === undefined
      ? {}
      : {
          refetchInterval: usageRefreshIntervalMs(refreshIntervalSeconds),
          refetchIntervalInBackground: true,
        }),
  });
}

/** Default options without background polling; prefer `createUsageSnapshotsQueryOptions`. */
export const usageSnapshotsQueryOptions = createUsageSnapshotsQueryOptions();
