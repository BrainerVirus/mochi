import { createFileRoute } from "@tanstack/react-router";

import { WidgetWindow } from "@/features/widget/components/widget-window";

export const Route = createFileRoute("/widget")({
  component: WidgetWindow,
});
