import { createFileRoute } from "@tanstack/react-router";

import { TrayPanel } from "@/features/tray/components/tray-panel/tray-panel";

export const Route = createFileRoute("/")({
  component: HomePage,
});

function HomePage() {
  return <TrayPanel />;
}
