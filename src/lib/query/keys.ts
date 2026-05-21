export const queryKeys = {
  appVersion: ["app", "version"] as const,
  usageSnapshots: ["usage", "snapshots"] as const,
  updateCheck: (channel: string) => ["update", "check", channel] as const,
};
