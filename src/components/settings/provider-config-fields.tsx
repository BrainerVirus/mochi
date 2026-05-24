import { AppSegmentedControl } from "@/components/ui/app-segmented-control";
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
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
    <FieldGroup className="gap-3">
      {entry.settingsFields.map((field) => renderProviderField(field, entry, config, onChange))}
    </FieldGroup>
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
    <Field key={field.key} className="flex-col gap-2">
      <FieldContent>
        <FieldLabel className="text-xs font-medium">{field.label}</FieldLabel>
      </FieldContent>
      <AppSegmentedControl
        items={[
          { id: "auto", label: "Auto" },
          { id: "manual", label: "Manual" },
          { id: "off", label: "Off" },
        ]}
        value={value}
        onValueChange={(source) => {
          if (source === "auto" || source === "manual" || source === "off") {
            onChange({ cookie_source: source });
          }
        }}
        rowHeight="h-8"
        stretchItems
      />
    </Field>
  );
}

function renderManualCookieField(
  field: ProviderCatalogEntry["settingsFields"][number],
  entry: ProviderCatalogEntry,
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  return (
    <Field key={field.key}>
      <FieldContent>
        <FieldLabel htmlFor={`${entry.id}-${field.key}`} className="text-xs font-medium">
          {field.label}
        </FieldLabel>
        <Input
          id={`${entry.id}-${field.key}`}
          className="h-8 font-mono text-xs"
          placeholder="oc_locale=en; auth=Fe26.2…"
          value={config.manual_cookie ?? ""}
          onChange={(event) => {
            onChange({ manual_cookie: event.target.value });
          }}
        />
        <FieldDescription className="text-[11px]">
          Paste the cookie string from browser DevTools, or set{" "}
          <code className="text-[10px]">
            MOCHI_{entry.id.toUpperCase().replace("-", "_")}_COOKIE
          </code>{" "}
          in your environment.
        </FieldDescription>
      </FieldContent>
    </Field>
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
    <Field key={field.key}>
      <FieldContent>
        <FieldLabel htmlFor={`${entry.id}-${field.key}`} className="text-xs font-medium">
          {field.label}
        </FieldLabel>
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
      </FieldContent>
    </Field>
  );
}

function renderTokenAccountField(
  field: ProviderCatalogEntry["settingsFields"][number],
  entry: ProviderCatalogEntry,
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  return (
    <Field key={field.key}>
      <FieldContent>
        <FieldLabel htmlFor={`${entry.id}-${field.key}`} className="text-xs font-medium">
          {field.label}
        </FieldLabel>
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
      </FieldContent>
    </Field>
  );
}

function renderHistoryWindowField(
  field: ProviderCatalogEntry["settingsFields"][number],
  entry: ProviderCatalogEntry,
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  return (
    <Field key={field.key} orientation="horizontal" className="items-center justify-between gap-3">
      <FieldContent className="min-w-0">
        <FieldLabel htmlFor={`${entry.id}-${field.key}`} className="text-xs font-medium">
          {field.label}
        </FieldLabel>
        <FieldDescription className="text-[11px]">Days of session cost history.</FieldDescription>
      </FieldContent>
      <Input
        id={`${entry.id}-${field.key}`}
        type="number"
        min={1}
        max={365}
        className="h-7 w-20 shrink-0 tabular-nums"
        value={config.history_window_days ?? 30}
        onChange={(event) => {
          const value = Number(event.target.value);
          if (Number.isFinite(value)) {
            onChange({ history_window_days: value });
          }
        }}
      />
    </Field>
  );
}

function renderRegionHostField(
  field: ProviderCatalogEntry["settingsFields"][number],
  entry: ProviderCatalogEntry,
  config: ProviderConfig,
  onChange: (patch: Partial<ProviderConfig>) => void,
) {
  return (
    <Field key={field.key}>
      <FieldContent>
        <FieldLabel htmlFor={`${entry.id}-${field.key}`} className="text-xs font-medium">
          {field.label}
        </FieldLabel>
        <Input
          id={`${entry.id}-${field.key}`}
          className="h-8"
          placeholder="api.example.com"
          value={config.region_host ?? ""}
          onChange={(event) => {
            onChange({ region_host: event.target.value });
          }}
        />
      </FieldContent>
    </Field>
  );
}
