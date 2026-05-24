import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";

import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { ProviderCatalogEntry } from "@/lib/schemas/provider-catalog";
import { ALL_PROVIDER_IDS, PROVIDER_LABELS, type MochiSettings } from "@/lib/schemas/settings";
import type { ProviderId } from "@/lib/schemas/usage";
import { getProviderCatalog, getProviderCredentialStatus } from "@/lib/tauri/commands";
import { cn } from "@/lib/utils";

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
    <div className="flex flex-col">
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
    </div>
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
      {showSeparator ? <Separator className="my-0" /> : null}
      <div className="flex flex-col gap-3 py-3">
        <div className="flex items-center justify-between gap-3">
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <span className="truncate text-sm font-medium">{PROVIDER_LABELS[provider]}</span>
              {entry ? (
                <Badge variant="outline" className="text-[10px]">
                  {entry.implementationStatus}
                </Badge>
              ) : null}
            </div>
            <div className="mt-0.5 flex flex-wrap items-center gap-1.5">
              {credentialStatus?.configured ? (
                <Badge variant="secondary" className="text-[10px]">
                  {credentialStatus.source ?? "Configured"}
                </Badge>
              ) : (
                <span className="text-muted-foreground text-[11px]">Not configured</span>
              )}
            </div>
          </div>
          <Switch
            checked={enabled}
            aria-label={`Toggle ${PROVIDER_LABELS[provider]}`}
            onCheckedChange={onToggle}
          />
        </div>

        {enabled && entry && entry.settingsFields.length > 0 ? (
          <div className="flex flex-col gap-3 pl-0.5">
            <ProviderConfigFields entry={entry} config={config} onChange={onConfigChange} />
          </div>
        ) : null}
      </div>
    </div>
  );
}

interface GeneralSettingsSectionProps {
  settings: MochiSettings;
  onChange: (patch: Partial<MochiSettings>) => void;
}

export function GeneralSettingsSection({ settings, onChange }: GeneralSettingsSectionProps) {
  return (
    <div className="flex flex-col gap-4">
      <SettingsField
        label="Refresh interval"
        description="How often Mochi checks usage (seconds)."
        htmlFor="refresh-interval"
      >
        <Input
          id="refresh-interval"
          type="number"
          min={30}
          max={86_400}
          className="h-8 w-28 tabular-nums"
          value={settings.refresh_interval_seconds}
          onChange={(event) => {
            const value = Number(event.target.value);
            if (Number.isFinite(value)) {
              onChange({ refresh_interval_seconds: value });
            }
          }}
        />
      </SettingsField>

      <SettingsField label="Update channel" description="Which release channel to follow.">
        <ToggleGroup
          type="single"
          value={settings.update_channel}
          onValueChange={(channel) => {
            if (channel === "stable" || channel === "unstable") {
              onChange({ update_channel: channel });
            }
          }}
          className="justify-start"
        >
          <ToggleGroupItem value="stable" size="sm">
            Stable
          </ToggleGroupItem>
          <ToggleGroupItem value="unstable" size="sm">
            Unstable
          </ToggleGroupItem>
        </ToggleGroup>
        {settings.update_channel === "unstable" ? (
          <p className="text-muted-foreground text-[11px]">
            Unstable builds may change frequently.
          </p>
        ) : null}
      </SettingsField>

      <div className="flex items-center justify-between gap-3">
        <div className="min-w-0 flex-1">
          <Label htmlFor="show-notifications" className="text-sm">
            Usage notifications
          </Label>
          <p className="text-muted-foreground text-[11px]">
            Soft alerts before providers hit hard limits.
          </p>
        </div>
        <Switch
          id="show-notifications"
          checked={settings.show_notifications}
          onCheckedChange={(checked) => {
            onChange({ show_notifications: checked });
          }}
        />
      </div>
    </div>
  );
}

function SettingsField({
  label,
  description,
  htmlFor,
  children,
}: {
  label: string;
  description?: string;
  htmlFor?: string;
  children: ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <Label htmlFor={htmlFor} className="text-sm">
        {label}
      </Label>
      {description ? (
        <p className={cn("text-muted-foreground text-[11px]", htmlFor ? "-mt-0.5" : undefined)}>
          {description}
        </p>
      ) : null}
      {children}
    </div>
  );
}
