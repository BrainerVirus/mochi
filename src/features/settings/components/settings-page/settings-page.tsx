import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

import { AppWindowShell } from "@/features/layout/components/app-window-shell";
import { SettingsPageContent } from "@/features/settings/components/settings-page-content";

const TRAY_PANEL_WINDOW_LABEL = "main";
const SETTINGS_WINDOW_LABEL = "settings";

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
