import { create } from "zustand";

import type { MochiSettings } from "@/lib/schemas/settings";
import { ProviderIdSchema, type ProviderId } from "@/lib/schemas/usage";
import { syncTrayUsage } from "@/lib/tauri/commands";

export type TraySelectedTab = "overview" | ProviderId;

export function readStoredTab(): TraySelectedTab {
  if (typeof window === "undefined") {
    return "overview";
  }

  // Read from Rust-injected initialization script (synchronous, no IPC)
  const globalWindow = window as unknown as Record<string, string | undefined>;
  const initialTab = globalWindow.__MOCHI_SELECTED_TAB__;
  if (typeof initialTab === "string") {
    if (initialTab === "overview") return "overview";
    const parsed = ProviderIdSchema.safeParse(initialTab);
    if (parsed.success) return parsed.data;
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
