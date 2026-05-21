import { useRef } from "react";

import { isTauriTrayPanel } from "@/lib/utils/tray-panel-height-sync";
import { markTrayTabFillPending } from "@/lib/utils/tray-tab-fill-scheduler";

import {
  formatTabFillActivationKey,
  nextTabFillActivationState,
} from "@/lib/utils/tray-tab-fill-activation";

/** Stable key that changes on every tray tab switch, even when returning to the same tab id. */
export function useTabFillActivationKey(activeTab: string): string {
  const stateRef = useRef({ tab: activeTab, generation: 0 });

  if (stateRef.current.tab !== activeTab) {
    if (isTauriTrayPanel()) {
      markTrayTabFillPending();
    }

    stateRef.current = nextTabFillActivationState(stateRef.current, activeTab);
  }

  return formatTabFillActivationKey(stateRef.current);
}
