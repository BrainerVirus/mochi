import type { ProviderCatalogEntry } from "@/lib/schemas/provider-catalog";
import type { ProviderConfig } from "@/lib/schemas/settings";

function providerHasCookieSource(entry: ProviderCatalogEntry): boolean {
  return entry.settingsFields.some((settingsField) => settingsField.kind === "cookie-source");
}

/** Fields entered only when cookie source is Manual (not Auto-detect or Off). */
function isCookieManualOnlyField(field: ProviderCatalogEntry["settingsFields"][number]): boolean {
  if (field.kind === "manual-cookie" || field.kind === "history-window") {
    return true;
  }

  return field.key === "token_accounts" || field.key === "workspace_id";
}

/** Manual cookie / session fields only appear when cookie source is manual. */
export function shouldShowProviderField(
  field: ProviderCatalogEntry["settingsFields"][number],
  config: ProviderConfig,
  entry: ProviderCatalogEntry,
): boolean {
  const cookieSource = config.cookie_source ?? "auto";
  const hasCookieSource = providerHasCookieSource(entry);

  if (field.kind === "cookie-source") {
    return true;
  }

  if (hasCookieSource && isCookieManualOnlyField(field)) {
    return cookieSource === "manual";
  }

  if (hasCookieSource && cookieSource === "off") {
    return false;
  }

  return true;
}
