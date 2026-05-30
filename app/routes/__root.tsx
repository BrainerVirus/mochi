import { createRootRoute } from "@tanstack/react-router";

import { RootComponent } from "@/components/layout/root-component";

export const Route = createRootRoute({
  component: RootComponent,
});
