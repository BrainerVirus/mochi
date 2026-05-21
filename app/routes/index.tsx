import { createFileRoute } from "@tanstack/react-router";

import { TrayPanel } from "@/components/tray/tray-panel";

export const Route = createFileRoute("/")({
  component: HomePage,
});

function HomePage() {
  return <TrayPanel />;
}
