import { invoke } from "@tauri-apps/api/core";

import { UpdateInfoSchema, type UpdateInfo } from "@/lib/schemas/usage";

export function appVersion(): Promise<string> {
  return invoke<string>("app_version");
}

export async function checkForUpdate(channel: string): Promise<UpdateInfo> {
  const result = await invoke<unknown>("check_for_update", { channel });
  return UpdateInfoSchema.parse(result);
}

export function installUpdate(): Promise<void> {
  return invoke<void>("install_update");
}
