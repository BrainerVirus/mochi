import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { ProviderCatalogEntry } from "@/lib/schemas/provider-catalog";
import type { ProviderConfig } from "@/lib/schemas/settings";

interface ProviderConfigFieldsProps {
  entry: ProviderCatalogEntry;
  config: ProviderConfig;
  onChange: (patch: Partial<ProviderConfig>) => void;
}

export function ProviderConfigFields({ entry, config, onChange }: ProviderConfigFieldsProps) {
  return (
    <>{entry.settingsFields.map((field) => renderProviderField(field, entry, config, onChange))}</>
  );
}

function renderProviderField(
  field: ProviderCatalogEntry["settingsFields"][number],
  entry: ProviderCatalogEntry,
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  if (field.kind === "cookie-source") {
    return renderCookieSourceField(field, config, onChange);
  }

  if (field.kind === "manual-cookie") {
    return renderManualCookieField(field, entry, config, onChange);
  }

  if (field.kind === "api-key" || field.key === "admin_api_key") {
    return renderApiKeyField(field, entry, config, onChange);
  }

  if (field.kind === "token-account") {
    return renderTokenAccountField(field, entry, config, onChange);
  }

  if (field.kind === "history-window") {
    return renderHistoryWindowField(field, entry, config, onChange);
  }

  if (field.kind === "region-host") {
    return renderRegionHostField(field, entry, config, onChange);
  }

  return null;
}

function renderCookieSourceField(
  field: ProviderCatalogEntry["settingsFields"][number],
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  const value = config.cookie_source ?? "auto";

  return (
    <div key={field.key} className="flex flex-col gap-2">
      <Label>{field.label}</Label>
      <div className="flex flex-wrap gap-2">
        {(
          [
            ["auto", "Auto"],
            ["manual", "Manual"],
            ["off", "Off"],
          ] as const
        ).map(([source, label]) => (
          <Button
            key={source}
            type="button"
            size="sm"
            variant={value === source ? "default" : "outline"}
            onClick={() => {
              onChange({ cookie_source: source });
            }}
          >
            {label}
          </Button>
        ))}
      </div>
    </div>
  );
}

function renderManualCookieField(
  field: ProviderCatalogEntry["settingsFields"][number],
  entry: ProviderCatalogEntry,
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  if (config.cookie_source === "off") {
    return null;
  }

  return (
    <div key={field.key} className="flex flex-col gap-2">
      <Label htmlFor={`${entry.id}-${field.key}`}>{field.label}</Label>
      <Input
        id={`${entry.id}-${field.key}`}
        placeholder="session=…; other=…"
        value={config.manual_cookie ?? ""}
        onChange={(event) => {
          onChange({ manual_cookie: event.target.value });
        }}
      />
      <p className="text-muted-foreground text-xs">
        Paste the HTTP Cookie header from browser DevTools, or set{" "}
        <code className="text-xs">MOCHI_{entry.id.toUpperCase()}_COOKIE</code> in your environment.
      </p>
    </div>
  );
}

function renderApiKeyField(
  field: ProviderCatalogEntry["settingsFields"][number],
  entry: ProviderCatalogEntry,
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  const value =
    field.key === "admin_api_key" ? (config.admin_api_key ?? "") : (config.api_key ?? "");

  return (
    <div key={field.key} className="flex flex-col gap-2">
      <Label htmlFor={`${entry.id}-${field.key}`}>{field.label}</Label>
      <Input
        id={`${entry.id}-${field.key}`}
        type="password"
        autoComplete="off"
        value={value}
        onChange={(event) => {
          if (field.key === "admin_api_key") {
            onChange({ admin_api_key: event.target.value });
          } else {
            onChange({ api_key: event.target.value });
          }
        }}
      />
    </div>
  );
}

function renderTokenAccountField(
  field: ProviderCatalogEntry["settingsFields"][number],
  entry: ProviderCatalogEntry,
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  return (
    <div key={field.key} className="flex flex-col gap-2">
      <Label htmlFor={`${entry.id}-${field.key}`}>{field.label}</Label>
      <Input
        id={`${entry.id}-${field.key}`}
        type="password"
        autoComplete="off"
        placeholder="GitHub OAuth token"
        value={config.token_account ?? ""}
        onChange={(event) => {
          onChange({ token_account: event.target.value });
        }}
      />
    </div>
  );
}

function renderHistoryWindowField(
  field: ProviderCatalogEntry["settingsFields"][number],
  entry: ProviderCatalogEntry,
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  return (
    <div key={field.key} className="flex flex-col gap-2">
      <Label htmlFor={`${entry.id}-${field.key}`}>{field.label} (days)</Label>
      <Input
        id={`${entry.id}-${field.key}`}
        type="number"
        min={1}
        max={365}
        value={config.history_window_days ?? 30}
        onChange={(event) => {
          const value = Number(event.target.value);
          if (Number.isFinite(value)) {
            onChange({ history_window_days: value });
          }
        }}
      />
    </div>
  );
}

function renderRegionHostField(
  field: ProviderCatalogEntry["settingsFields"][number],
  entry: ProviderCatalogEntry,
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  return (
    <div key={field.key} className="flex flex-col gap-2">
      <Label htmlFor={`${entry.id}-${field.key}`}>{field.label}</Label>
      <Input
        id={`${entry.id}-${field.key}`}
        placeholder="api.example.com"
        value={config.region_host ?? ""}
        onChange={(event) => {
          onChange({ region_host: event.target.value });
        }}
      />
    </div>
  );
}
