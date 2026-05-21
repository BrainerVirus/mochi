import { queryOptions } from "@tanstack/react-query";

import { getUsageSnapshots } from "@/lib/tauri/commands";

import { queryKeys } from "./keys";

export const usageSnapshotsQueryOptions = queryOptions({
  queryKey: queryKeys.usageSnapshots,
  queryFn: getUsageSnapshots,
});
