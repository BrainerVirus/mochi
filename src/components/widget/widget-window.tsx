import { RefreshCwIcon } from "lucide-react";

import { MochiMascot } from "@/components/mascot/mochi-mascot";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { UsageCard } from "@/components/usage/usage-card";
import { useRefreshProvider } from "@/hooks/use-tray-events";
import { useUsageData } from "@/hooks/use-usage-data";
import { widgetDensityClasses } from "@/lib/utils/widget-density";

export function WidgetWindow() {
  const density = widgetDensityClasses("compact");
  const { data, error, isError, isPending, isSuccess, refetch, isFetching } = useUsageData();
  const refreshProvider = useRefreshProvider();
  const isRefreshing = isFetching || refreshProvider.isPending;

  return (
    <main className="bg-background text-foreground min-h-screen">
      <section
        className={`mx-auto flex min-h-screen w-full max-w-[480px] min-w-[280px] flex-col ${density.root}`}
      >
        <header className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <MochiMascot state={isError ? "warning" : "normal"} className="size-8" />
            <div>
              <p className="text-muted-foreground text-[10px] font-medium tracking-[0.2em] uppercase">
                Mochi
              </p>
              <h1 className={density.title}>Widget</h1>
            </div>
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
        </header>

        <Card className={`rounded-mochi shadow-sm ${density.card}`}>
          <CardHeader className="p-0">
            <CardTitle className={density.title}>Providers</CardTitle>
          </CardHeader>
          <CardContent className={`flex flex-col ${density.card}`}>
            <WidgetUsagePanel
              data={data}
              error={error}
              isError={isError}
              isPending={isPending}
              isSuccess={isSuccess}
            />
          </CardContent>
        </Card>
      </section>
    </main>
  );
}

interface WidgetUsagePanelProps {
  data: ReturnType<typeof useUsageData>["data"];
  error: ReturnType<typeof useUsageData>["error"];
  isError: boolean;
  isPending: boolean;
  isSuccess: boolean;
}

function WidgetUsagePanel({ data, error, isError, isPending, isSuccess }: WidgetUsagePanelProps) {
  if (isPending) {
    return (
      <output className="text-muted-foreground block text-center text-xs">
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
      <p className="text-muted-foreground text-center text-xs">
        Enable providers in settings to see usage here.
      </p>
    );
  }

  if (isSuccess && data !== undefined) {
    return (
      <ul className="flex w-full flex-col gap-2">
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
