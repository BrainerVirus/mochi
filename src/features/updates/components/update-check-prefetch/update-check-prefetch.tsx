import { useUpdateCheck } from "@/features/updates/hooks/use-update-install/use-update-install";

export function UpdateCheckPrefetch() {
  useUpdateCheck();
  return null;
}
