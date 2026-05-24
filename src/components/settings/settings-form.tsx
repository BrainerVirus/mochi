import { useState } from "react";

import { ScrollFadeRegion } from "@/components/tray/scroll-fade-region";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { AppSegmentedControl } from "@/components/ui/app-segmented-control";
import { useSaveSettings, useSettings } from "@/hooks/use-tray-events";
import type { MochiSettings } from "@/lib/schemas/settings";
import { trayPanelSpacing } from "@/lib/utils/tray-panel-spacing";

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
      <div className={`${trayPanelSpacing.contentX} py-6`}>
        <Alert variant="destructive">
          <AlertTitle>Could not load settings</AlertTitle>
          <AlertDescription>{error?.message ?? "Unknown error"}</AlertDescription>
        </Alert>
      </div>
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
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div
        className={`border-border shrink-0 ${trayPanelSpacing.contentX} border-b pt-3 pb-2`}
        data-settings-tab-strip
      >
        <AppSegmentedControl
          items={[...SETTINGS_TABS]}
          value={activeTab}
          onValueChange={setActiveTab}
          variant="page-tabs"
          rowHeight="h-9"
          stretchItems
        />
      </div>

      <ScrollFadeRegion
        orientation="vertical"
        className="min-h-0 flex-1"
        scrollClassName="overscroll-y-contain"
      >
        <div className={`${trayPanelSpacing.contentX} ${trayPanelSpacing.contentTop}`}>
          {activeTab === "general" ? (
            isLoading ? (
              <SettingsLoadingState />
            ) : (
              <GeneralSettingsSection settings={settings} onChange={patchSettings} />
            )
          ) : isLoading ? (
            <SettingsLoadingState />
          ) : (
            <ProviderSettingsSection settings={settings} onChange={patchSettings} />
          )}
        </div>
      </ScrollFadeRegion>

      <p
        className={`text-muted-foreground shrink-0 ${trayPanelSpacing.contentX} ${trayPanelSpacing.footerBottom} py-2 text-center text-[11px]`}
      >
        {isSaving ? "Saving…" : "Changes save automatically"}
      </p>
    </div>
  );
}

function SettingsLoadingState() {
  return (
    <output className="text-muted-foreground block py-6 text-center text-xs">
      Loading settings…
    </output>
  );
}
