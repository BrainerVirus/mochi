import { createFileRoute } from "@tanstack/react-router";

import { TrayPanel } from "@/components/tray/tray-panel";

export const Route = createFileRoute("/")({
  // Tray panel uses Tauri invoke; skip SSR so SPA shell prerender and dev SSR stay stable.
  ssr: false,
  component: HomePage,
});

function HomePage() {
  return <TrayPanel />;
}
