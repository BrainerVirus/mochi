import { APP_WINDOW_LABEL } from "@/lib/tauri/app-window";
import { TRAY_PANEL_WINDOW_LABEL } from "@/lib/tauri/tray-panel-window";

export const WIDGET_WINDOW_LABEL = "widget";

export type InitialWindowRoute = "/" | "/settings" | "/widget";

/** Maps Tauri webview labels to the client route each window should show after shell boot. */
export function initialRouteForWindowLabel(label: string): InitialWindowRoute {
  switch (label) {
    case APP_WINDOW_LABEL:
      return "/settings";
    case WIDGET_WINDOW_LABEL:
      return "/widget";
    case TRAY_PANEL_WINDOW_LABEL:
    default:
      return "/";
  }
}

/** True when the webview loaded the packaged SPA shell instead of a deep route. */
export function shouldNavigateFromPackagedShell(pathname: string): boolean {
  if (pathname === "/" || pathname === "/index.html") {
    return true;
  }

  return pathname.endsWith(".html");
}

/**
 * Synchronously applies the `#/path` boot hash injected by the Rust window
 * builder (`initial_app_url_for_path`). Runs before router creation so first
 * paint renders the requested route — the async pending-route handoff stays
 * as fallback for windows booted at the plain shell URL.
 */
export function consumeBootHashRoute(): string | null {
  if (typeof window === "undefined") {
    return null;
  }
  const { hash, search } = window.location;
  if (!hash.startsWith("#/")) {
    return null;
  }
  const target = hash.slice(1);
  window.history.replaceState(null, "", `${target}${search}`);
  return target;
}
