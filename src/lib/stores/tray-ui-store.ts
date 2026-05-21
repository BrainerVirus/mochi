import { create } from "zustand";

import { ProviderIdSchema, type ProviderId } from "@/lib/schemas/usage";

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
