import { useQuery } from "@tanstack/react-query";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { ALL_PROVIDER_IDS, PROVIDER_LABELS, type MochiSettings } from "@/lib/schemas/settings";
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
    <div className="flex flex-col gap-4">
      {ALL_PROVIDER_IDS.map((provider) => {
        const entry = catalogById.get(provider);
        const enabled = settings.enabled_providers.includes(provider);

        return (
          <Card key={provider} className="rounded-mochi shadow-sm">
            <CardHeader className="gap-3">
              <div className="flex items-start justify-between gap-3">
                <div className="flex flex-col gap-1">
                  <CardTitle className="text-base">{PROVIDER_LABELS[provider]}</CardTitle>
                  <CardDescription className="flex flex-wrap items-center gap-2">
                    {entry ? <Badge variant="outline">{entry.implementationStatus}</Badge> : null}
                    {credentialStatus[provider]?.configured ? (
                      <Badge variant="secondary">
                        {credentialStatus[provider]?.source ?? "Credentials detected"}
                      </Badge>
                    ) : (
                      <Badge variant="outline">Not configured</Badge>
                    )}
                  </CardDescription>
                </div>
                <Switch
                  checked={enabled}
                  aria-label={`Toggle ${PROVIDER_LABELS[provider]}`}
                  onCheckedChange={() => {
                    const nextEnabled = enabled
                      ? settings.enabled_providers.filter((item) => item !== provider)
                      : [...settings.enabled_providers, provider];
                    onChange({ enabled_providers: nextEnabled });
                  }}
                />
              </div>
            </CardHeader>

            {enabled && entry && entry.settingsFields.length > 0 ? (
              <CardContent className="flex flex-col gap-4 border-t pt-4">
                <ProviderConfigFields
                  entry={entry}
                  config={settings.provider_configs[provider] ?? {}}
                  onChange={(patch) => {
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
              </CardContent>
            ) : null}
          </Card>
        );
      })}
    </div>
  );
}

interface GeneralSettingsSectionProps {
  settings: MochiSettings;
  onChange: (patch: Partial<MochiSettings>) => void;
}

export function GeneralSettingsSection({ settings, onChange }: GeneralSettingsSectionProps) {
  return (
    <Card className="rounded-mochi shadow-sm">
      <CardHeader>
        <CardTitle>Refresh & updates</CardTitle>
        <CardDescription>
          Control how often Mochi refreshes usage and which update channel you follow.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="flex flex-col gap-2">
          <Label htmlFor="refresh-interval">Refresh interval (seconds)</Label>
          <Input
            id="refresh-interval"
            type="number"
            min={30}
            max={86_400}
            value={settings.refresh_interval_seconds}
            onChange={(event) => {
              const value = Number(event.target.value);
              if (Number.isFinite(value)) {
                onChange({ refresh_interval_seconds: value });
              }
            }}
          />
        </div>

        <div className="flex flex-col gap-2">
          <Label>Update channel</Label>
          <div className="flex flex-wrap gap-2">
            {(["stable", "unstable"] as const).map((channel) => (
              <Button
                key={channel}
                type="button"
                size="sm"
                variant={settings.update_channel === channel ? "default" : "outline"}
                onClick={() => {
                  onChange({ update_channel: channel });
                }}
              >
                {channel === "stable" ? "Stable" : "Unstable"}
              </Button>
            ))}
          </div>
          {settings.update_channel === "unstable" ? (
            <Badge variant="secondary">Unstable builds may change frequently.</Badge>
          ) : null}
        </div>

        <div className="flex items-center justify-between gap-3">
          <div className="flex flex-col gap-1">
            <Label htmlFor="show-notifications">Usage notifications</Label>
            <p className="text-muted-foreground text-xs">
              Show soft alerts before providers hit hard limits.
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
      </CardContent>
    </Card>
  );
}
