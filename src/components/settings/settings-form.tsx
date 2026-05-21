import { Link } from "@tanstack/react-router";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useSaveSettings, useSettings } from "@/hooks/use-tray-events";
import type { MochiSettings } from "@/lib/schemas/settings";

import { GeneralSettingsSection, ProviderSettingsSection } from "./settings-sections";

export function SettingsForm() {
  const { data, error, isError, isPending } = useSettings();
  const saveSettings = useSaveSettings();

  if (isPending) {
    return (
      <output className="text-muted-foreground block text-center text-sm">Loading settings…</output>
    );
  }

  if (isError) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Could not load settings</AlertTitle>
        <AlertDescription>{error?.message ?? "Unknown error"}</AlertDescription>
      </Alert>
    );
  }

  if (!data) {
    return null;
  }

  return (
    <SettingsEditor
      settings={data}
      isSaving={saveSettings.isPending}
      onSave={(next) => {
        saveSettings.mutate(next);
      }}
    />
  );
}

interface SettingsEditorProps {
  settings: MochiSettings;
  isSaving: boolean;
  onSave: (settings: MochiSettings) => void;
}

function SettingsEditor({ settings, isSaving, onSave }: SettingsEditorProps) {
  function patchSettings(patch: Partial<MochiSettings>) {
    onSave({ ...settings, ...patch });
  }

  return (
    <Tabs defaultValue="general" className="flex flex-col gap-4">
      <TabsList>
        <TabsTrigger value="general">General</TabsTrigger>
        <TabsTrigger value="providers">Providers</TabsTrigger>
      </TabsList>

      <TabsContent value="general" className="flex flex-col gap-4">
        <GeneralSettingsSection settings={settings} onChange={patchSettings} />
      </TabsContent>

      <TabsContent value="providers" className="flex flex-col gap-4">
        <ProviderSettingsSection settings={settings} onChange={patchSettings} />
      </TabsContent>

      <div className="flex items-center justify-between gap-3">
        <Button variant="outline" asChild>
          <Link to="/">Back to tray panel</Link>
        </Button>
        <p className="text-muted-foreground text-xs">
          {isSaving ? "Saving settings…" : "Changes save automatically."}
        </p>
      </div>
    </Tabs>
  );
}
