import { useQuery } from "@tanstack/react-query";

import { AppSegmentedControl } from "@/components/ui/app-segmented-control";
import { Badge } from "@/components/ui/badge";
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import type { ProviderCatalogEntry } from "@/lib/schemas/provider-catalog";
import { ALL_PROVIDER_IDS, PROVIDER_LABELS, type MochiSettings } from "@/lib/schemas/settings";
import type { ProviderId } from "@/lib/schemas/usage";
import { getProviderCatalog, getProviderCredentialStatus } from "@/lib/tauri/commands";

import { ProviderConfigFields } from "./provider-config-fields";

interface ProviderSettingsSectionProps {
  settings: MochiSettings;
  onChange: (patch: Partial<MochiSettings>) => void;
}

export function ProviderSettingsSection({ settings, onChange }: ProviderSettingsSectionProps) {
  const { data: catalog = [] } = useQuery({
    queryKey: ["provider-catalog"],
    queryFn: getProviderCatalog,
    staleTime: Number.POSITIVE_INFINITY,
  });

  const { data: credentialStatus = {} } = useQuery({
    queryKey: ["provider-credential-status"],
    queryFn: getProviderCredentialStatus,
    staleTime: 30_000,
  });

  const catalogById = new Map(catalog.map((entry) => [entry.id, entry]));

  return (
    <FieldGroup className="gap-0">
      {ALL_PROVIDER_IDS.map((provider, index) => (
        <ProviderSettingsRow
          key={provider}
          provider={provider}
          showSeparator={index > 0}
          entry={catalogById.get(provider)}
          enabled={settings.enabled_providers.includes(provider)}
          config={settings.provider_configs[provider] ?? {}}
          credentialStatus={credentialStatus[provider]}
          onToggle={() => {
            const enabled = settings.enabled_providers.includes(provider);
            const nextEnabled = enabled
              ? settings.enabled_providers.filter((item) => item !== provider)
              : [...settings.enabled_providers, provider];
            onChange({ enabled_providers: nextEnabled });
          }}
          onConfigChange={(patch) => {
            onChange({
              provider_configs: {
                ...settings.provider_configs,
                [provider]: {
                  ...settings.provider_configs[provider],
                  ...patch,
                },
              },
            });
          }}
        />
      ))}
    </FieldGroup>
  );
}

function ProviderSettingsRow({
  provider,
  showSeparator,
  entry,
  enabled,
  config,
  credentialStatus,
  onToggle,
  onConfigChange,
}: {
  provider: ProviderId;
  showSeparator: boolean;
  entry?: ProviderCatalogEntry;
  enabled: boolean;
  config: MochiSettings["provider_configs"][ProviderId];
  credentialStatus?: { configured?: boolean; source?: string };
  onToggle: () => void;
  onConfigChange: (patch: Partial<MochiSettings["provider_configs"][ProviderId]>) => void;
}) {
  return (
    <div>
      {showSeparator ? <Separator /> : null}
      <Field orientation="horizontal" className="items-start justify-between gap-3 py-2.5">
        <FieldContent className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <FieldLabel className="text-sm font-medium">{PROVIDER_LABELS[provider]}</FieldLabel>
            {entry ? (
              <Badge variant="outline" className="text-[10px]">
                {entry.implementationStatus}
              </Badge>
            ) : null}
          </div>
          {credentialStatus?.configured ? (
            <Badge variant="secondary" className="mt-1 text-[10px]">
              {credentialStatus.source ?? "Configured"}
            </Badge>
          ) : (
            <FieldDescription className="text-[11px]">Not configured</FieldDescription>
          )}
        </FieldContent>
        <Switch
          checked={enabled}
          aria-label={`Toggle ${PROVIDER_LABELS[provider]}`}
          onCheckedChange={onToggle}
        />
      </Field>

      {enabled && entry && entry.settingsFields.length > 0 ? (
        <div className="flex flex-col gap-3 pb-2.5">
          <ProviderConfigFields entry={entry} config={config} onChange={onConfigChange} />
        </div>
      ) : null}
    </div>
  );
}

interface GeneralSettingsSectionProps {
  settings: MochiSettings;
  onChange: (patch: Partial<MochiSettings>) => void;
}

export function GeneralSettingsSection({ settings, onChange }: GeneralSettingsSectionProps) {
  return (
    <FieldGroup className="gap-0">
      <Field orientation="horizontal" className="items-center justify-between gap-3 py-2.5">
        <FieldContent className="min-w-0">
          <FieldLabel htmlFor="refresh-interval" className="text-sm font-medium">
            Refresh interval
          </FieldLabel>
          <FieldDescription className="text-[11px]">
            How often Mochi checks usage (seconds).
          </FieldDescription>
        </FieldContent>
        <Input
          id="refresh-interval"
          type="number"
          min={30}
          max={86_400}
          className="h-7 w-20 shrink-0 tabular-nums"
          value={settings.refresh_interval_seconds}
          onChange={(event) => {
            const value = Number(event.target.value);
            if (Number.isFinite(value)) {
              onChange({ refresh_interval_seconds: value });
            }
          }}
        />
      </Field>

      <Separator />

      <Field className="flex-col gap-2 py-2.5">
        <FieldContent>
          <FieldLabel className="text-sm font-medium">Update channel</FieldLabel>
          <FieldDescription className="text-[11px]">
            Which release channel to follow.
          </FieldDescription>
        </FieldContent>
        <AppSegmentedControl
          items={[
            { id: "stable", label: "Stable" },
            { id: "unstable", label: "Unstable" },
          ]}
          value={settings.update_channel}
          onValueChange={(channel) => {
            if (channel === "stable" || channel === "unstable") {
              onChange({ update_channel: channel });
            }
          }}
          rowHeight="h-8"
          stretchItems
        />
        {settings.update_channel === "unstable" ? (
          <FieldDescription className="text-[11px]">
            Unstable builds may change frequently.
          </FieldDescription>
        ) : null}
      </Field>

      <Separator />

      <Field orientation="horizontal" className="items-center justify-between gap-3 py-2.5">
        <FieldContent className="min-w-0">
          <FieldLabel htmlFor="show-notifications" className="text-sm font-medium">
            Usage notifications
          </FieldLabel>
          <FieldDescription className="text-[11px]">
            Soft alerts before providers hit hard limits.
          </FieldDescription>
        </FieldContent>
        <Switch
          id="show-notifications"
          checked={settings.show_notifications}
          onCheckedChange={(checked) => {
            onChange({ show_notifications: checked });
          }}
        />
      </Field>
    </FieldGroup>
  );
}
