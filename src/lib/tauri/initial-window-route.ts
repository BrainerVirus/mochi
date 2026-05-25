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
