export function usageSnapshotsEmptyMessage(enabledProviderCount: number): string {
  if (enabledProviderCount === 0) {
    return "All providers are disabled. Enable at least one in settings to see usage.";
  }

  return "No provider usage snapshots yet. Enable providers in settings to get started.";
}
