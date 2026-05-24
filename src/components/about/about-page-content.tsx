import { useQuery } from "@tanstack/react-query";

import { MochiChibi } from "@/components/mascot/mochi-chibi";
import { queryKeys } from "@/lib/query/keys";
import { appVersion } from "@/lib/tauri/commands";
import { trayPanelSpacing } from "@/lib/utils/tray-panel-spacing";

export function AboutPageContent() {
  const { data: version = "…" } = useQuery({
    queryKey: queryKeys.appVersion,
    queryFn: appVersion,
    staleTime: Number.POSITIVE_INFINITY,
  });

  return (
    <div
      className={`text-foreground flex min-h-svh flex-col items-center justify-center ${trayPanelSpacing.contentX} py-8 text-center`}
    >
      <MochiChibi className="mb-4 size-[4.5rem]" />
      <h1 className="text-base font-semibold tracking-tight">Mochi</h1>
      <p className="text-muted-foreground mt-1 max-w-[16rem] text-xs leading-relaxed">
        Soft alerts before hard limits.
      </p>
      <p className="text-muted-foreground mt-4 text-[11px] tabular-nums">Version {version}</p>
    </div>
  );
}
