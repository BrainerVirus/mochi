import { SettingsForm } from "@/components/settings/settings-form";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export function SettingsPageContent() {
  return (
    <section className="mx-auto flex min-h-full w-full max-w-[720px] flex-col gap-6 p-6">
      <Card className="rounded-mochi shadow-sm">
        <CardHeader>
          <CardDescription className="font-medium tracking-[0.2em] uppercase">
            Mochi
          </CardDescription>
          <CardTitle className="text-3xl font-semibold">Settings</CardTitle>
          <CardDescription>
            Configure refresh behavior, update channel, notifications, and enabled providers.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <SettingsForm />
        </CardContent>
      </Card>
    </section>
  );
}
