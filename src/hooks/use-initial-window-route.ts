import { useNavigate } from "@tanstack/react-router";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect } from "react";

import {
  initialRouteForWindowLabel,
  shouldNavigateFromPackagedShell,
} from "@/lib/tauri/initial-window-route";

/** Navigates packaged Tauri webviews from the static shell to their window-specific route. */
export function useInitialWindowRoute() {
  const navigate = useNavigate();

  useEffect(() => {
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
      return;
    }

    try {
      const label = getCurrentWebviewWindow().label;
      const pathname = window.location.pathname;
      if (!shouldNavigateFromPackagedShell(pathname)) {
        return;
      }

      const target = initialRouteForWindowLabel(label);
      if (pathname === target) {
        return;
      }

      void navigate({ to: target, replace: true });
    } catch {
      // Ignore when the webview API is unavailable (e.g. unit tests).
    }
  }, [navigate]);
}
