import { createFileRoute } from "@tanstack/react-router";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

import { AppWindowShell } from "@/components/layout/app-window-shell";
import { SettingsPageContent } from "@/components/settings/settings-page-content";

const TRAY_PANEL_WINDOW_LABEL = "main";
const SETTINGS_WINDOW_LABEL = "settings";

export const Route = createFileRoute("/settings")({
  // Settings depend on Tauri invoke; skip SSR so server HTML matches client mount.
  ssr: false,
  component: SettingsPage,
});

export function SettingsPage() {
  if (isDedicatedAppWindow()) {
    return (
      <AppWindowShell>
        <SettingsPageContent />
      </AppWindowShell>
    );
  }

  if (isTrayPanelWindow()) {
    return <SettingsPageContent />;
  }

  return (
    <AppWindowShell>
      <SettingsPageContent />
    </AppWindowShell>
  );
}

function isDedicatedAppWindow(): boolean {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return false;
  }

  try {
    const label = getCurrentWebviewWindow().label;
    return label === SETTINGS_WINDOW_LABEL;
  } catch {
    return false;
  }
}

function isTrayPanelWindow(): boolean {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return false;
  }

  try {
    return getCurrentWebviewWindow().label === TRAY_PANEL_WINDOW_LABEL;
  } catch {
    return false;
  }
}
