import { createFileRoute } from "@tanstack/react-router";

import { WidgetWindow } from "@/components/widget/widget-window";

export const Route = createFileRoute("/widget")({
  component: WidgetWindow,
});
