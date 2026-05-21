import { useRef } from "react";

import {
  formatTabFillActivationKey,
  nextTabFillActivationState,
} from "@/lib/utils/tray-tab-fill-activation";

/** Stable key that changes on every tray tab switch, even when returning to the same tab id. */
export function useTabFillActivationKey(activeTab: string): string {
  const stateRef = useRef({ tab: activeTab, generation: 0 });

  if (stateRef.current.tab !== activeTab) {
    stateRef.current = nextTabFillActivationState(stateRef.current, activeTab);
  }

  return formatTabFillActivationKey(stateRef.current);
}
