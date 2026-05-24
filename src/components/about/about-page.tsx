import { AboutPageContent } from "@/components/about/about-page-content";
import { AppWindowShell } from "@/components/layout/app-window-shell";

export function AboutPage() {
  return (
    <AppWindowShell variant="about">
      <AboutPageContent />
    </AppWindowShell>
  );
}
