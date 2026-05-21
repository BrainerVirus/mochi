import { useQuery } from "@tanstack/react-query";

import { settingsQueryOptions } from "@/lib/query/settings";
import { createUsageSnapshotsQueryOptions } from "@/lib/query/usage-snapshots";

export function useUsageData() {
  const { data: settings } = useQuery(settingsQueryOptions);

  return useQuery(createUsageSnapshotsQueryOptions(settings?.refresh_interval_seconds));
}
