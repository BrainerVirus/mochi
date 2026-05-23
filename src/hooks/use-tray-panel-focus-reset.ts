import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, type RefObject } from "react";

import { clearTrayPanelFocus, clearTrayPanelFocusDeferred } from "@/lib/utils/tray-panel-focus";
import { isTauriTrayPanel } from "@/lib/utils/tray-panel-height-sync";

export function useTrayPanelFocusReset(panelRef: RefObject<HTMLElement | null>) {
  useEffect(() => {
    let unlistenFocus: (() => void) | undefined;

    if (isTauriTrayPanel()) {
      const panelRoot = () => panelRef.current ?? document;

      void getCurrentWebviewWindow()
        .onFocusChanged(({ payload: focused }) => {
          if (focused) {
            clearTrayPanelFocusDeferred(panelRoot());
            return;
          }

          clearTrayPanelFocus(panelRoot());
        })
        .then((unlisten) => {
          unlistenFocus = unlisten;
        });

      clearTrayPanelFocusDeferred(panelRoot());
    }

    return () => {
      unlistenFocus?.();
    };
  }, [panelRef]);
}
