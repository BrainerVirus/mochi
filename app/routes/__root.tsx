import { createRootRoute } from "@tanstack/react-router";

import { RootComponent } from "@/components/layout/root-component";

import appCss from "@/styles/index.css?url";

export const Route = createRootRoute({
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1" },
      { title: "Mochi" },
    ],
    links: [{ rel: "stylesheet", href: appCss }],
  }),
  component: RootComponent,
});
