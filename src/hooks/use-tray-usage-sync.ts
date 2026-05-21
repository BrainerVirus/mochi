import { useEffect } from "react";

import { useUsageData } from "@/hooks/use-usage-data";
import { syncTrayUsage } from "@/lib/tauri/commands";

export function useTrayUsageSync() {
  const { data, isSuccess } = useUsageData();

  useEffect(() => {
    if (!isSuccess) {
      return;
    }

    void syncTrayUsage();
  }, [data, isSuccess]);
}
