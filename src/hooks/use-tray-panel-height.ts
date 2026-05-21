import { useEffect, type RefObject } from "react";

import { clampTrayPanelHeight } from "@/lib/utils/tray-panel-layout";
import { setTrayPanelHeight } from "@/lib/tauri/commands";

function isTauriTrayPanel(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Resizes the native tray popover to match measured content, capped at viewport max.
 */
export function useTrayPanelHeight(contentRef: RefObject<HTMLElement | null>) {
  useEffect(() => {
    if (!isTauriTrayPanel()) {
      return undefined;
    }

    const element = contentRef.current;
    if (!element) {
      return undefined;
    }

    let frame = 0;

    const syncHeight = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const viewportHeight = window.screen.availHeight;
        const height = clampTrayPanelHeight(element.scrollHeight, viewportHeight);
        void setTrayPanelHeight(height);
      });
    };

    const resizeObserver = new ResizeObserver(syncHeight);
    resizeObserver.observe(element);
    syncHeight();

    window.addEventListener("resize", syncHeight);

    return () => {
      cancelAnimationFrame(frame);
      resizeObserver.disconnect();
      window.removeEventListener("resize", syncHeight);
    };
  }, [contentRef]);
}
