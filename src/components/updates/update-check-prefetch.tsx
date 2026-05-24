import { useUpdateCheck } from "@/hooks/use-update-install";

export function UpdateCheckPrefetch() {
  useUpdateCheck();
  return null;
}
