import { useEffect, type RefObject } from "react";

import { setTrayPanelHeight } from "@/lib/tauri/commands";
import {
  clampTrayPanelHeight,
  measureTrayPanelLayoutHeight,
} from "@/lib/utils/tray-panel-layout";

function isTauriTrayPanel(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

interface TrayPanelHeightRefs {
  contentRef: RefObject<HTMLElement | null>;
  footerRef: RefObject<HTMLElement | null>;
}

/**
 * Resizes the native tray popover to match measured content + footer, capped at viewport max.
 */
export function useTrayPanelHeight({ contentRef, footerRef }: TrayPanelHeightRefs) {
  useEffect(() => {
    if (!isTauriTrayPanel()) {
      return undefined;
    }

    const content = contentRef.current;
    if (!content) {
      return undefined;
    }

    let frame = 0;

    const syncHeight = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const viewportHeight = window.screen.availHeight;
        const layoutHeight = measureTrayPanelLayoutHeight(content, footerRef.current);
        const height = clampTrayPanelHeight(layoutHeight, viewportHeight);
        void setTrayPanelHeight(height);
      });
    };

    const resizeObserver = new ResizeObserver(syncHeight);
    resizeObserver.observe(content);
    const footer = footerRef.current;
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
  }, [contentRef, footerRef]);
}
