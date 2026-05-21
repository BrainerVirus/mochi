import { queryOptions } from "@tanstack/react-query";

import { getUsageSnapshots } from "@/lib/tauri/commands";

import { queryKeys } from "./keys";
import { usageRefreshIntervalMs } from "./usage-refetch-interval";

export function createUsageSnapshotsQueryOptions(refreshIntervalSeconds?: number) {
  return queryOptions({
    queryKey: queryKeys.usageSnapshots,
    queryFn: getUsageSnapshots,
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
