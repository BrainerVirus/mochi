import { create } from "zustand";

import type { MochiSettings } from "@/lib/schemas/settings";
import { type ProviderId } from "@/lib/schemas/usage";
import { syncTrayUsage } from "@/lib/tauri/commands";
import { parseTrayTabChange } from "@/lib/utils/tray-tab-selection";

export type TraySelectedTab = "overview" | ProviderId;

export function readStoredTab(): TraySelectedTab {
  if (typeof window === "undefined") {
    return "overview";
  }

  // Read from Rust-injected initialization script (synchronous, no IPC)
  const globalWindow = window as unknown as Record<string, string | undefined>;
  const initialTab = globalWindow.__MOCHI_SELECTED_TAB__;
  if (typeof initialTab === "string") {
    return parseTrayTabChange(initialTab);
  }

  return "overview";
}

interface TrayUiStore {
  selectedTab: TraySelectedTab;
  setSelectedTab: (tab: TraySelectedTab) => void;
}

export const useTrayUiStore = create<TrayUiStore>((set) => ({
  selectedTab: readStoredTab(),
  setSelectedTab: (tab) => {
    set({ selectedTab: tab });
  },
}));

export function resolveValidTraySelection(
  selected: TraySelectedTab,
  enabledProviders: ProviderId[],
): TraySelectedTab {
  if (selected === "overview") {
    return "overview";
  }

  return enabledProviders.includes(selected) ? selected : "overview";
}

export function currentTraySelection(): TraySelectedTab {
  return useTrayUiStore.getState().selectedTab;
}

export function syncCurrentTrayUsage(
  settings: Pick<MochiSettings, "enabled_providers">,
): Promise<void> {
  const selected = currentTraySelection();
  const validSelection = resolveValidTraySelection(selected, settings.enabled_providers);
  if (validSelection !== selected) {
    useTrayUiStore.getState().setSelectedTab(validSelection);
  }
  return syncTrayUsage(validSelection);
}
