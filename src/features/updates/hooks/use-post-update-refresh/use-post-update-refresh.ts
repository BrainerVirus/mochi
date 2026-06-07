import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { queryKeys } from "@/lib/query/keys";
import { isTauriRuntime } from "@/lib/tauri/runtime";
import { consumePostUpdateRefreshPending } from "@/lib/updates/release-notes-cache";

export function usePostUpdateRefresh() {
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!isTauriRuntime() || !consumePostUpdateRefreshPending()) {
      return;
    }

    void queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots });
    void queryClient.invalidateQueries({ queryKey: queryKeys.settings });
    void queryClient.invalidateQueries({ queryKey: ["update", "check"] });
  }, [queryClient]);
}
