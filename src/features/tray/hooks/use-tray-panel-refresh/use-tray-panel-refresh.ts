import { useCallback, useState } from "react";

import { useQueryClient } from "@tanstack/react-query";

import { queryKeys } from "@/lib/query/keys";
import { refreshAllProviders } from "@/lib/tauri/commands";

export function useTrayPanelRefresh() {
  const queryClient = useQueryClient();
  const [isRefreshingAll, setIsRefreshingAll] = useState(false);

  const refreshAll = useCallback(async () => {
    if (isRefreshingAll) return;
    setIsRefreshingAll(true);
    try {
      await refreshAllProviders();
    } catch {
      void queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots });
    } finally {
      setIsRefreshingAll(false);
    }
  }, [isRefreshingAll, queryClient]);

  return { refreshAll, isRefreshingAll };
}
