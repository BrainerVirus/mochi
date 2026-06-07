import { create } from "zustand";

type PanelDensity = "compact" | "normal" | "expanded";

interface UiStore {
  selectedProvider: string | null;
  panelDensity: PanelDensity;
  setSelectedProvider: (provider: string | null) => void;
  setPanelDensity: (density: PanelDensity) => void;
}

export const useUiStore = create<UiStore>((set) => ({
  selectedProvider: null,
  panelDensity: "normal",
  setSelectedProvider: (provider) => set({ selectedProvider: provider }),
  setPanelDensity: (density) => set({ panelDensity: density }),
}));
