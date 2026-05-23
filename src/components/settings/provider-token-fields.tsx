import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { ProviderCatalogEntry } from "@/lib/schemas/provider-catalog";
import type { ProviderConfig, TokenAccountData } from "@/lib/schemas/settings";

function emptyTokenAccounts(): TokenAccountData {
  return {
    version: 1,
    accounts: [
      {
        id: crypto.randomUUID(),
        label: "",
        token: "",
      },
    ],
    activeIndex: 0,
  };
}

export function TokenAccountsField({
  field,
  entry,
  config,
  onChange,
}: {
  field: ProviderCatalogEntry["settingsFields"][number];
  entry: ProviderCatalogEntry;
  config: ProviderConfig;
  onChange: (patch: Partial<ProviderConfig>) => void;
}) {
  if (config.cookie_source === "off") {
    return null;
  }

  const data = config.token_accounts ?? emptyTokenAccounts();
  const activeIndex = Math.min(data.activeIndex, Math.max(data.accounts.length - 1, 0));
  const account = data.accounts[activeIndex] ?? {
    id: crypto.randomUUID(),
    label: "",
    token: "",
  };

  return (
    <div key={field.key} className="flex flex-col gap-3">
      <Label>{field.label}</Label>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${entry.id}-token-label`}>Account label</Label>
        <Input
          id={`${entry.id}-token-label`}
          placeholder="zen"
          value={account.label}
          onChange={(event) => {
            const accounts = [...data.accounts];
            accounts[activeIndex] = { ...account, label: event.target.value };
            onChange({
              token_accounts: {
                ...data,
                accounts,
                activeIndex,
              },
            });
          }}
        />
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${entry.id}-token-cookie`}>Cookie</Label>
        <Input
          id={`${entry.id}-token-cookie`}
          type="password"
          autoComplete="off"
          placeholder="oc_locale=en; auth=Fe26.2…"
          value={account.token}
          onChange={(event) => {
            const accounts = [...data.accounts];
            accounts[activeIndex] = { ...account, token: event.target.value };
            onChange({
              token_accounts: {
                ...data,
                accounts,
                activeIndex,
              },
              cookie_source: config.cookie_source ?? "manual",
            });
          }}
        />
      </div>
      <p className="text-muted-foreground text-xs">
        Store labeled OpenCode Go cookies like CodexBar token accounts. The active account is used
        when cookie source is Manual or Auto cannot find browser cookies.
      </p>
    </div>
  );
}

export function WorkspaceIdField({
  field,
  entry,
  config,
  onChange,
}: {
  field: ProviderCatalogEntry["settingsFields"][number];
  entry: ProviderCatalogEntry;
  config: ProviderConfig;
  onChange: (patch: Partial<ProviderConfig>) => void;
}) {
  return (
    <div key={field.key} className="flex flex-col gap-2">
      <Label htmlFor={`${entry.id}-${field.key}`}>{field.label}</Label>
      <Input
        id={`${entry.id}-${field.key}`}
        placeholder="wrk_… or https://opencode.ai/workspace/wrk_…/go"
        value={config.workspace_id ?? config.token_account ?? ""}
        onChange={(event) => {
          onChange({ workspace_id: event.target.value });
        }}
      />
    </div>
  );
}
