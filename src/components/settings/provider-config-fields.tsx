import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { ProviderCatalogEntry } from "@/lib/schemas/provider-catalog";
import type { ProviderConfig } from "@/lib/schemas/settings";

import { TokenAccountsField, WorkspaceIdField } from "./provider-token-fields";

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

/** Manual cookie / session-cost fields only appear when cookie source is manual. */
export function shouldShowProviderField(
  field: ProviderCatalogEntry["settingsFields"][number],
  config: ProviderConfig,
): boolean {
  const cookieSource = config.cookie_source ?? "auto";

  if (field.kind === "cookie-source") {
    return true;
  }

  if (field.kind === "manual-cookie" || field.kind === "history-window") {
    return cookieSource === "manual";
  }

  if (cookieSource === "off") {
    return false;
  }

  return true;
}

function renderProviderField(
  field: ProviderCatalogEntry["settingsFields"][number],
  entry: ProviderCatalogEntry,
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  if (!shouldShowProviderField(field, config)) {
    return null;
  }

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
    if (field.key === "token_accounts") {
      return (
        <TokenAccountsField
          key={field.key}
          field={field}
          entry={entry}
          config={config}
          onChange={onChange}
        />
      );
    }

    return renderTokenAccountField(field, entry, config, onChange);
  }

  if (field.kind === "history-window") {
    return renderHistoryWindowField(field, entry, config, onChange);
  }

  if (field.kind === "region-host") {
    if (field.key === "workspace_id") {
      return (
        <WorkspaceIdField
          key={field.key}
          field={field}
          entry={entry}
          config={config}
          onChange={onChange}
        />
      );
    }

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
    <div key={field.key} className="flex flex-col gap-1.5">
      <Label className="text-xs">{field.label}</Label>
      <ToggleGroup
        type="single"
        value={value}
        onValueChange={(source) => {
          if (source === "auto" || source === "manual" || source === "off") {
            onChange({ cookie_source: source });
          }
        }}
        className="justify-start"
      >
        <ToggleGroupItem value="auto" size="sm">
          Auto
        </ToggleGroupItem>
        <ToggleGroupItem value="manual" size="sm">
          Manual
        </ToggleGroupItem>
        <ToggleGroupItem value="off" size="sm">
          Off
        </ToggleGroupItem>
      </ToggleGroup>
    </div>
  );
}

function renderManualCookieField(
  field: ProviderCatalogEntry["settingsFields"][number],
  entry: ProviderCatalogEntry,
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  return (
    <div key={field.key} className="flex flex-col gap-1.5">
      <Label htmlFor={`${entry.id}-${field.key}`} className="text-xs">
        {field.label}
      </Label>
      <Input
        id={`${entry.id}-${field.key}`}
        className="h-8 font-mono text-xs"
        placeholder="oc_locale=en; auth=Fe26.2…"
        value={config.manual_cookie ?? ""}
        onChange={(event) => {
          onChange({ manual_cookie: event.target.value });
        }}
      />
      <p className="text-muted-foreground text-[11px]">
        Paste the cookie string from browser DevTools, or set{" "}
        <code className="text-[10px]">MOCHI_{entry.id.toUpperCase().replace("-", "_")}_COOKIE</code>{" "}
        in your environment.
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
    <div key={field.key} className="flex flex-col gap-1.5">
      <Label htmlFor={`${entry.id}-${field.key}`} className="text-xs">
        {field.label}
      </Label>
      <Input
        id={`${entry.id}-${field.key}`}
        type="password"
        autoComplete="off"
        className="h-8"
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
    <div key={field.key} className="flex flex-col gap-1.5">
      <Label htmlFor={`${entry.id}-${field.key}`} className="text-xs">
        {field.label}
      </Label>
      <Input
        id={`${entry.id}-${field.key}`}
        type="password"
        autoComplete="off"
        className="h-8"
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
    <div key={field.key} className="flex flex-col gap-1.5">
      <Label htmlFor={`${entry.id}-${field.key}`} className="text-xs">
        {field.label} (days)
      </Label>
      <Input
        id={`${entry.id}-${field.key}`}
        type="number"
        min={1}
        max={365}
        className="h-8 w-24 tabular-nums"
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
    <div key={field.key} className="flex flex-col gap-1.5">
      <Label htmlFor={`${entry.id}-${field.key}`} className="text-xs">
        {field.label}
      </Label>
      <Input
        id={`${entry.id}-${field.key}`}
        className="h-8"
        placeholder="api.example.com"
        value={config.region_host ?? ""}
        onChange={(event) => {
          onChange({ region_host: event.target.value });
        }}
      />
    </div>
  );
}
