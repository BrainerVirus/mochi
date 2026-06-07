import { SettingsForm } from "@/features/settings/components/settings-form";

export function SettingsPageContent() {
  return (
    <div className="text-foreground flex min-h-0 flex-1 flex-col overflow-hidden">
      <SettingsForm />
    </div>
  );
}
