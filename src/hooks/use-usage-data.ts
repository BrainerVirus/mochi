import { useQuery } from "@tanstack/react-query";

import { usageSnapshotsQueryOptions } from "@/lib/query/usage-snapshots";

export function useUsageData() {
  return useQuery(usageSnapshotsQueryOptions);
}
