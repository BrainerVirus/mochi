import { useEffect } from "react";

import { useSettings } from "@/features/tray/hooks/use-tray-events";
import {
  syncCurrentTrayUsage,
  useTrayUiStore,
} from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { useUsageData } from "@/features/usage/hooks/use-usage-data/use-usage-data";
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

    // Runs after settings-changed reconcile updates the settings/usage caches.
    void syncCurrentTrayUsage(settings);
  }, [data, isSuccess, selectedTab, settings]);
}
