import { mutationOptions } from "@tanstack/react-query";

import type { ProviderId } from "@/lib/schemas/usage";
import { refreshProvider } from "@/lib/tauri/commands";

import { queryKeys } from "@/lib/query/keys";

export function refreshProviderMutationOptions() {
  return mutationOptions({
    mutationFn: (provider: ProviderId) => refreshProvider(provider),
    meta: {
      invalidates: queryKeys.usageSnapshots,
    },
  });
}
