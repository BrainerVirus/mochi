import { createRootRoute } from "@tanstack/react-router";

import { RootComponent } from "@/features/layout/components/root-component";

export const Route = createRootRoute({
  component: RootComponent,
});
