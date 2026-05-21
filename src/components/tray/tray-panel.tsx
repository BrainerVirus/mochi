import { Link } from "@tanstack/react-router";
import { RefreshCwIcon, SettingsIcon } from "lucide-react";

import { MochiMascot } from "@/components/mascot/mochi-mascot";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { UsageCard } from "@/components/usage/usage-card";
import { useRefreshProvider, useSettings } from "@/hooks/use-tray-events";
import { useUsageData } from "@/hooks/use-usage-data";
import { usageSnapshotsEmptyMessage } from "@/lib/utils/usage-snapshots-empty-message";

export function TrayPanel() {
  const { data: settings } = useSettings();
  const { data, error, isError, isPending, isSuccess, refetch, isFetching } = useUsageData();
  const refreshProvider = useRefreshProvider();

  const isRefreshing = isFetching || refreshProvider.isPending;

  return (
    <main className="bg-background text-foreground min-h-full">
      <section className="mx-auto flex w-full max-w-[360px] flex-col gap-4 p-4">
        <header className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <MochiMascot state={isError ? "warning" : "normal"} className="size-10" />
            <div>
              <p className="text-xs font-medium tracking-[0.2em] uppercase">Mochi</p>
              <h1 className="text-lg font-semibold">Usage overview</h1>
            </div>
          </div>
          <Button variant="ghost" size="icon-sm" asChild>
            <Link to="/settings" aria-label="Open settings">
              <SettingsIcon data-icon="inline-start" />
            </Link>
          </Button>
        </header>

        <Card className="rounded-mochi shadow-sm">
          <CardHeader className="flex flex-row items-center justify-between gap-2">
            <div>
              <CardTitle className="text-base">Providers</CardTitle>
              <CardDescription>Soft alerts before hard limits.</CardDescription>
            </div>
            <Button
              variant="outline"
              size="sm"
              disabled={isRefreshing}
              onClick={() => {
                void refetch();
              }}
            >
              <RefreshCwIcon data-icon="inline-start" />
              Refresh
            </Button>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            <UsageSnapshotsPanel
              data={data}
              error={error}
              isError={isError}
              isPending={isPending}
              isSuccess={isSuccess}
              enabledProviderCount={settings?.enabled_providers.length ?? 0}
            />
          </CardContent>
        </Card>

        <Separator />

        <p className="text-muted-foreground text-center text-xs">
          Left-click the tray icon to reopen this panel. Right-click for refresh, settings, and
          updates.
        </p>
      </section>
    </main>
  );
}

interface UsageSnapshotsPanelProps {
  data: ReturnType<typeof useUsageData>["data"];
  error: ReturnType<typeof useUsageData>["error"];
  isError: boolean;
  isPending: boolean;
  isSuccess: boolean;
  enabledProviderCount: number;
}

function UsageSnapshotsPanel({
  data,
  error,
  isError,
  isPending,
  isSuccess,
  enabledProviderCount,
}: UsageSnapshotsPanelProps) {
  if (isPending) {
    return (
      <output className="text-muted-foreground block text-center text-sm">
        Loading provider usage…
      </output>
    );
  }

  if (isError) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Could not load usage</AlertTitle>
        <AlertDescription>{error?.message ?? "Unknown error"}</AlertDescription>
      </Alert>
    );
  }

  if (isSuccess && data !== undefined && data.length === 0) {
    return (
      <p className="text-muted-foreground text-center text-sm">
        {usageSnapshotsEmptyMessage(enabledProviderCount)}
      </p>
    );
  }

  if (isSuccess && data !== undefined) {
    return (
      <ul className="flex w-full flex-col gap-3">
        {data.map((snapshot) => (
          <li key={snapshot.provider}>
            <UsageCard snapshot={snapshot} />
          </li>
        ))}
      </ul>
    );
  }

  return null;
}
