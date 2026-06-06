import { useEffect } from "react";

import { useSettings } from "@/hooks/use-tray-events";
import { useUsageData } from "@/hooks/use-usage-data";
import { syncCurrentTrayUsage, useTrayUiStore } from "@/lib/stores/tray-ui-store";
import { syncTrayUsage } from "@/lib/tauri/commands";

export function useTrayUsageSync() {
  const { data, isSuccess } = useUsageData();
  const { data: settings } = useSettings();
  const selectedTab = useTrayUiStore((state) => state.selectedTab);

  // Tab changes and tray open: update menu bar from Rust cache immediately (no live fetch).
  useEffect(() => {
    void syncTrayUsage(selectedTab);
  }, [selectedTab]);

  useEffect(() => {
    if (!isSuccess || !settings) {
      return;
    }

    void syncCurrentTrayUsage(settings);
  }, [data, isSuccess, selectedTab, settings]);
}
