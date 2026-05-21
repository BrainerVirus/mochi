import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { useEffect, useRef, type RefObject } from "react";

import {
  isTauriTrayPanel,
  observeTrayPanelHeight,
  runTrayPanelTabHeightAnimation,
} from "@/lib/utils/tray-panel-height-sync";

gsap.registerPlugin(useGSAP);

/**
 * Resizes the native tray popover to match measured column height, capped at viewport max.
 * Tab switches morph height with GSAP; other layout changes sync immediately.
 */
export function useTrayPanelHeight(layoutRef: RefObject<HTMLElement | null>, activeTab: string) {
  const lastHeightRef = useRef<number | null>(null);
  const isTabAnimatingRef = useRef(false);
  const heightTweenRef = useRef<gsap.core.Tween | null>(null);
  const isInitialTabRef = useRef(true);

  useEffect(() => {
    if (!isTauriTrayPanel()) {
      return undefined;
    }

    const layout = layoutRef.current;
    if (!layout) {
      return undefined;
    }

    return observeTrayPanelHeight(layout, lastHeightRef, isTabAnimatingRef);
  }, [layoutRef]);

  useGSAP(
    () => {
      if (!isTauriTrayPanel()) {
        return undefined;
      }

      const layout = layoutRef.current;
      if (!layout) {
        return undefined;
      }

      return runTrayPanelTabHeightAnimation(layout, {
        lastHeightRef,
        isTabAnimatingRef,
        heightTweenRef,
        isInitialTabRef,
      });
    },
    { dependencies: [activeTab], scope: layoutRef, revertOnUpdate: true },
  );
}
