import { useEffect, type RefObject } from "react";

import { setTrayPanelHeight } from "@/lib/tauri/commands";
import {
  clampTrayPanelHeight,
  measureTrayPanelLayoutHeight,
} from "@/lib/utils/tray-panel-layout";

function isTauriTrayPanel(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Resizes the native tray popover to match measured column height, capped at viewport max.
 */
export function useTrayPanelHeight(layoutRef: RefObject<HTMLElement | null>) {
  useEffect(() => {
    if (!isTauriTrayPanel()) {
      return undefined;
    }

    const layout = layoutRef.current;
    if (!layout) {
      return undefined;
    }

    let frame = 0;

    const syncHeight = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const viewportHeight = window.screen.availHeight;
        const layoutHeight = measureTrayPanelLayoutHeight(layout);
        const height = clampTrayPanelHeight(layoutHeight, viewportHeight);
        void setTrayPanelHeight(height);
      });
    };

    const resizeObserver = new ResizeObserver(syncHeight);
    resizeObserver.observe(layout);

    const content = layout.querySelector("[data-tray-panel-content]");
    if (content) {
      resizeObserver.observe(content);
    }

    const footer = layout.querySelector("[data-tray-panel-footer]");
    if (footer) {
      resizeObserver.observe(footer);
    }

    syncHeight();

    window.addEventListener("resize", syncHeight);

    return () => {
      cancelAnimationFrame(frame);
      resizeObserver.disconnect();
      window.removeEventListener("resize", syncHeight);
    };
  }, [layoutRef]);
}
