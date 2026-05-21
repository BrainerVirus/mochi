import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  ALL_PROVIDER_IDS,
  PROVIDER_LABELS,
  type MochiSettings,
  type UpdateChannel,
} from "@/lib/schemas/settings";

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
            {(["stable", "unstable"] as UpdateChannel[]).map((channel) => (
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

interface ProviderSettingsSectionProps {
  settings: MochiSettings;
  onChange: (patch: Partial<MochiSettings>) => void;
}

export function ProviderSettingsSection({ settings, onChange }: ProviderSettingsSectionProps) {
  function toggleProvider(provider: MochiSettings["enabled_providers"][number]) {
    const enabled = settings.enabled_providers.includes(provider)
      ? settings.enabled_providers.filter((item) => item !== provider)
      : [...settings.enabled_providers, provider];

    onChange({ enabled_providers: enabled });
  }

  return (
    <Card className="rounded-mochi shadow-sm">
      <CardHeader>
        <CardTitle>Enabled providers</CardTitle>
        <CardDescription>
          Choose which v1 providers appear in the tray panel and widget.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {ALL_PROVIDER_IDS.map((provider) => {
          const enabled = settings.enabled_providers.includes(provider);

          return (
            <div
              key={provider}
              className="border-border flex items-center justify-between gap-3 rounded-lg border p-3"
            >
              <div className="flex flex-col gap-1">
                <span className="text-sm font-medium">{PROVIDER_LABELS[provider]}</span>
                <span className="text-muted-foreground text-xs capitalize">{provider}</span>
              </div>
              <Switch
                checked={enabled}
                aria-label={`Toggle ${PROVIDER_LABELS[provider]}`}
                onCheckedChange={() => {
                  toggleProvider(provider);
                }}
              />
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}
