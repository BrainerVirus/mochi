import { useCallback, useRef, useState } from "react";

import { useQueryClient } from "@tanstack/react-query";

import { queryKeys } from "@/lib/query/keys";
import { refreshAllProviders } from "@/lib/tauri/commands";

export function useTrayPanelRefresh() {
  const queryClient = useQueryClient();
  const [isRefreshingAll, setIsRefreshingAll] = useState(false);
  const isRefreshingRef = useRef(false);

  const refreshAll = useCallback(async () => {
    if (isRefreshingRef.current) return;
    isRefreshingRef.current = true;
    setIsRefreshingAll(true);
    try {
      await refreshAllProviders();
    } catch {
      void queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots });
    } finally {
      isRefreshingRef.current = false;
      setIsRefreshingAll(false);
    }
  }, [queryClient]);

  return { refreshAll, isRefreshingAll };
}
