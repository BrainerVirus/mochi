import { mutationOptions, queryOptions } from "@tanstack/react-query";

import type { MochiSettings } from "@/lib/schemas/settings";
import { getSettings, saveSettings } from "@/lib/tauri/commands";

import { queryKeys } from "./keys";

export const settingsQueryOptions = queryOptions({
  queryKey: queryKeys.settings,
  queryFn: getSettings,
});

export function saveSettingsMutationOptions() {
  return mutationOptions({
    mutationFn: (settings: MochiSettings) => saveSettings(settings),
    meta: {
      invalidates: queryKeys.settings,
    },
  });
}
