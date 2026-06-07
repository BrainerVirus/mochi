import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
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

function TokenAccountInputs({
  entry,
  account,
  activeIndex,
  data,
  config,
  onChange,
}: {
  entry: ProviderCatalogEntry;
  account: TokenAccountData["accounts"][number];
  activeIndex: number;
  data: TokenAccountData;
  config: ProviderConfig;
  onChange: (patch: Partial<ProviderConfig>) => void;
}) {
  return (
    <>
      <Field>
        <FieldContent>
          <FieldLabel htmlFor={`${entry.id}-token-label`} className="text-xs">
            Account label
          </FieldLabel>
          <Input
            id={`${entry.id}-token-label`}
            className="h-8"
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
        </FieldContent>
      </Field>
      <Field>
        <FieldContent>
          <FieldLabel htmlFor={`${entry.id}-token-cookie`} className="text-xs">
            Cookie
          </FieldLabel>
          <Input
            id={`${entry.id}-token-cookie`}
            type="password"
            autoComplete="off"
            className="h-8 font-mono text-xs"
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
        </FieldContent>
      </Field>
    </>
  );
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
    <FieldGroup key={field.key} className="gap-3">
      <FieldLabel className="text-xs font-medium">{field.label}</FieldLabel>
      <TokenAccountInputs
        entry={entry}
        account={account}
        activeIndex={activeIndex}
        data={data}
        config={config}
        onChange={onChange}
      />
      <FieldDescription className="text-[11px]">
        Store labeled OpenCode Go cookies like CodexBar token accounts. The active account is used
        when cookie source is Manual or Auto cannot find browser cookies.
      </FieldDescription>
    </FieldGroup>
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
    <Field key={field.key}>
      <FieldContent>
        <FieldLabel htmlFor={`${entry.id}-${field.key}`} className="text-xs font-medium">
          {field.label}
        </FieldLabel>
        <Input
          id={`${entry.id}-${field.key}`}
          className="h-8"
          placeholder="wrk_… or https://opencode.ai/workspace/wrk_…/go"
          value={config.workspace_id ?? config.token_account ?? ""}
          onChange={(event) => {
            onChange({ workspace_id: event.target.value });
          }}
        />
      </FieldContent>
    </Field>
  );
}
