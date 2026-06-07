import { create } from "zustand";

import type { MochiSettings } from "@/lib/schemas/settings";
import { ProviderIdSchema, type ProviderId } from "@/lib/schemas/usage";
import { syncTrayUsage } from "@/lib/tauri/commands";

export type TraySelectedTab = "overview" | ProviderId;

const STORAGE_KEY = "mochi-tray-selected-tab";

function readStoredTab(): TraySelectedTab {
  if (typeof window === "undefined") {
    return "overview";
  }

  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw === "overview") {
      return "overview";
    }

    const parsed = ProviderIdSchema.safeParse(raw);
    if (parsed.success) {
      return parsed.data;
    }
  } catch {
    // localStorage unavailable (private browsing, etc.)
  }

  return "overview";
}

function persistTab(tab: TraySelectedTab) {
  if (typeof window === "undefined") {
    return;
  }

  try {
    localStorage.setItem(STORAGE_KEY, tab);
  } catch {
    // ignore quota / privacy mode errors
  }
}

interface TrayUiStore {
  selectedTab: TraySelectedTab;
  setSelectedTab: (tab: TraySelectedTab) => void;
}

export const useTrayUiStore = create<TrayUiStore>((set) => ({
  selectedTab: readStoredTab(),
  setSelectedTab: (tab) => {
    persistTab(tab);
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
