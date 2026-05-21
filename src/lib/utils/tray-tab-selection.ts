import { ProviderIdSchema, type ProviderId } from "@/lib/schemas/usage";
import type { TraySelectedTab } from "@/lib/stores/tray-ui-store";

export function parseTrayTabChange(value: string): TraySelectedTab {
  if (value === "overview") {
    return "overview";
  }

  const parsed = ProviderIdSchema.safeParse(value);
  return parsed.success ? parsed.data : "overview";
}

export function isProviderTab(tab: TraySelectedTab): tab is ProviderId {
  return tab !== "overview";
}
