import type { TraySelectedTab } from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { ProviderIdSchema, type ProviderId } from "@/lib/schemas/usage";

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
