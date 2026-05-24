import { useQuery } from "@tanstack/react-query";

import { MochiMark } from "@/components/mascot/mochi-mark";
import { queryKeys } from "@/lib/query/keys";
import { appVersion } from "@/lib/tauri/commands";

export function AboutPageContent() {
  const { data: version = "…" } = useQuery({
    queryKey: queryKeys.appVersion,
    queryFn: appVersion,
    staleTime: Number.POSITIVE_INFINITY,
  });

  return (
    <div className="flex min-h-svh flex-col items-center justify-center px-6 py-8 text-center">
      <MochiMark state="all-good" className="mb-4 size-16" />
      <h1 className="text-lg font-semibold tracking-tight">Mochi</h1>
      <p className="text-muted-foreground mt-1 max-w-[16rem] text-xs leading-relaxed">
        Soft alerts before hard limits.
      </p>
      <p className="text-muted-foreground mt-4 text-[11px] tabular-nums">Version {version}</p>
    </div>
  );
}
