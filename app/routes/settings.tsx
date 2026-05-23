import { createFileRoute } from "@tanstack/react-router";

import { SettingsPage } from "@/components/settings/settings-page";

export const Route = createFileRoute("/settings")({
  // Settings depend on Tauri invoke; skip SSR so server HTML matches client mount.
  ssr: false,
  component: SettingsPage,
});
