import { createFileRoute } from "@tanstack/react-router";

import { AboutPageContent } from "@/components/about/about-page-content";
import { AppWindowShell } from "@/components/layout/app-window-shell";

export const Route = createFileRoute("/about")({
  ssr: false,
  component: AboutPage,
});

function AboutPage() {
  return (
    <AppWindowShell>
      <AboutPageContent />
    </AppWindowShell>
  );
}
