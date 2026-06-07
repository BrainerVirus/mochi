import { AboutPageContent } from "@/features/about/components/about-page-content";
import { AppWindowShell } from "@/features/layout/components/app-window-shell";

export function AboutPage() {
  return (
    <AppWindowShell variant="about">
      <AboutPageContent />
    </AppWindowShell>
  );
}
