import { createFileRoute } from "@tanstack/react-router";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

import { AppWindowShell } from "@/components/layout/app-window-shell";
import { SettingsForm } from "@/components/settings/settings-form";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

const TRAY_PANEL_WINDOW_LABEL = "main";
const SETTINGS_WINDOW_LABEL = "settings";

export const Route = createFileRoute("/settings")({
  // Settings depend on Tauri invoke; skip SSR so server HTML matches client mount.
  ssr: false,
  component: SettingsPage,
});

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

function SettingsPage() {
  const content = (
    <section className="mx-auto flex min-h-full w-full max-w-[720px] flex-col gap-6 p-6">
      <Card className="rounded-mochi shadow-sm">
        <CardHeader>
          <CardDescription className="font-medium tracking-[0.2em] uppercase">
            Mochi
          </CardDescription>
          <CardTitle className="text-3xl font-semibold">Settings</CardTitle>
          <CardDescription>
            Configure refresh behavior, update channel, notifications, and enabled providers.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <SettingsForm />
        </CardContent>
      </Card>
    </section>
  );

  if (isDedicatedAppWindow()) {
    return <AppWindowShell>{content}</AppWindowShell>;
  }

  if (isTrayPanelWindow()) {
    return content;
  }

  return <AppWindowShell>{content}</AppWindowShell>;
}
