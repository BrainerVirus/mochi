import { DEFAULT_MOCHI_SETTINGS, type MochiSettings } from "@/lib/schemas/settings";

interface SettingsQueryState {
  data: MochiSettings | undefined;
  isPending: boolean;
  isError: boolean;
}

type SettingsFormState =
  | { kind: "error" }
  | { kind: "editor"; settings: MochiSettings; isLoading: boolean };

export function resolveSettingsFormState(state: SettingsQueryState): SettingsFormState {
  if (state.isError) {
    return { kind: "error" };
  }

  return {
    kind: "editor",
    settings: state.data ?? DEFAULT_MOCHI_SETTINGS,
    isLoading: state.isPending && !state.data,
  };
}
