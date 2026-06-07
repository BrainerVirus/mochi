import { createFileRoute } from "@tanstack/react-router";

import { AboutPage } from "@/features/about/components/about-page";

export const Route = createFileRoute("/about")({
  component: AboutPage,
});
