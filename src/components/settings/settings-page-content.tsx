import { SettingsForm } from "@/components/settings/settings-form";

export function SettingsPageContent() {
  return (
    <div className="flex min-h-svh flex-col px-4 py-5">
      <header className="mb-4 flex flex-col gap-0.5">
        <h1 className="text-base font-semibold tracking-tight">Settings</h1>
        <p className="text-muted-foreground text-xs">
          Refresh behavior, updates, notifications, and providers.
        </p>
      </header>
      <SettingsForm />
    </div>
  );
}
