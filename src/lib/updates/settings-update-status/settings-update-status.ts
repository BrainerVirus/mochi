export interface SettingsUpdateStatusInput {
  updateAvailable: boolean;
  version: string | null;
  isFetching: boolean;
  installPhase: "idle" | "downloading" | "installing" | "error";
  installPending: boolean;
  installError: string | null;
}

export function resolveSettingsUpdateStatusLabel({
  updateAvailable,
  version,
  isFetching,
  installPhase,
  installPending,
  installError,
}: SettingsUpdateStatusInput): string {
  if (installError) {
    return installError;
  }

  if (installPending && installPhase === "installing") {
    return "Installing update…";
  }

  if (installPending && installPhase === "downloading") {
    return "Downloading update…";
  }

  if (updateAvailable) {
    return `Update available${version ? ` (v${version})` : ""}`;
  }

  if (isFetching) {
    return "Checking for updates…";
  }

  return "You're up to date";
}

export function shouldShowSettingsInstallButton({
  updateAvailable,
  installPending,
}: Pick<SettingsUpdateStatusInput, "updateAvailable" | "installPending">): boolean {
  return updateAvailable && !installPending;
}
