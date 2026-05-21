import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { useRef, type RefObject } from "react";

import {
  runTrayTabContentEnterAnimation,
  setTrayTabContentVisible,
} from "@/lib/utils/tray-tab-content-animation";

gsap.registerPlugin(useGSAP);

/**
 * Animates tab content sections (stats, links, overview blocks) on tab switch.
 * Skips the first render; coordinates timing with {@link useTrayPanelHeight}.
 */
export function useTrayTabTransition(
  containerRef: RefObject<HTMLElement | null>,
  activeTab: string,
) {
  const isInitialTabRef = useRef(true);

  useGSAP(
    () => {
      const container = containerRef.current;
      if (!container) {
        return undefined;
      }

      if (isInitialTabRef.current) {
        isInitialTabRef.current = false;
        setTrayTabContentVisible(container);
        return undefined;
      }

      return runTrayTabContentEnterAnimation(container);
    },
    { dependencies: [activeTab], scope: containerRef, revertOnUpdate: true },
  );
}
