import { useState } from "react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { AppSegmentedControl } from "@/components/ui/app-segmented-control";
import { useSaveSettings, useSettings } from "@/hooks/use-tray-events";
import type { MochiSettings } from "@/lib/schemas/settings";

import { resolveSettingsFormState } from "./settings-form-state";
import { GeneralSettingsSection, ProviderSettingsSection } from "./settings-sections";

const SETTINGS_TABS = [
  { id: "general", label: "General" },
  { id: "providers", label: "Providers" },
] as const;

export function SettingsForm() {
  const { data, error, isError, isPending } = useSettings();
  const saveSettings = useSaveSettings();
  const view = resolveSettingsFormState({ data, isPending, isError });

  if (view.kind === "error") {
    return (
      <Alert variant="destructive">
        <AlertTitle>Could not load settings</AlertTitle>
        <AlertDescription>{error?.message ?? "Unknown error"}</AlertDescription>
      </Alert>
    );
  }

  return (
    <SettingsEditor
      settings={view.settings}
      isLoading={view.isLoading}
      isSaving={saveSettings.isPending}
      onSave={(next) => {
        saveSettings.mutate(next);
      }}
    />
  );
}

interface SettingsEditorProps {
  settings: MochiSettings;
  isLoading: boolean;
  isSaving: boolean;
  onSave: (settings: MochiSettings) => void;
}

function SettingsEditor({ settings, isLoading, isSaving, onSave }: SettingsEditorProps) {
  const [activeTab, setActiveTab] = useState<string>("general");

  function patchSettings(patch: Partial<MochiSettings>) {
    onSave({ ...settings, ...patch });
  }

  return (
    <div className="flex flex-col gap-4">
      <AppSegmentedControl
        items={[...SETTINGS_TABS]}
        value={activeTab}
        onValueChange={setActiveTab}
      />

      {activeTab === "general" ? (
        isLoading ? (
          <output className="text-muted-foreground block py-6 text-center text-sm">
            Loading settings…
          </output>
        ) : (
          <GeneralSettingsSection settings={settings} onChange={patchSettings} />
        )
      ) : isLoading ? (
        <output className="text-muted-foreground block py-6 text-center text-sm">
          Loading settings…
        </output>
      ) : (
        <ProviderSettingsSection settings={settings} onChange={patchSettings} />
      )}

      <p className="text-muted-foreground text-center text-[11px]">
        {isSaving ? "Saving…" : "Changes save automatically"}
      </p>
    </div>
  );
}
