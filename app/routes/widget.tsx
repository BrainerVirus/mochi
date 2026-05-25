import { createFileRoute } from "@tanstack/react-router";

import { WidgetWindow } from "@/components/widget/widget-window";

export const Route = createFileRoute("/widget")({
  ssr: false,
  component: WidgetPage,
});

function WidgetPage() {
  return <WidgetWindow />;
}
