import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useState } from "react";

import { queryKeys } from "@/lib/query/keys";
import type { ProviderId } from "@/lib/schemas/usage";
import { refreshProvider, syncTrayUsage } from "@/lib/tauri/commands";

interface UseTrayPanelRefreshOptions {
  enabledProviders: ProviderId[];
  refetch: () => Promise<unknown>;
}

export function useTrayPanelRefresh({ enabledProviders, refetch }: UseTrayPanelRefreshOptions) {
  const queryClient = useQueryClient();
  const [isRefreshingAll, setIsRefreshingAll] = useState(false);

  const refreshAll = useCallback(async () => {
    if (isRefreshingAll) {
      return;
    }

    setIsRefreshingAll(true);
    try {
      if (enabledProviders.length > 0) {
        await Promise.allSettled(enabledProviders.map((provider) => refreshProvider(provider)));
      }
      await queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots });
      await syncTrayUsage();
      await refetch();
    } finally {
      setIsRefreshingAll(false);
    }
  }, [enabledProviders, isRefreshingAll, queryClient, refetch]);

  return {
    refreshAll,
    isRefreshingAll,
  };
}
