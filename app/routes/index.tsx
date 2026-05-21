import { createFileRoute } from "@tanstack/react-router";

import { MochiMascot } from "@/components/mascot/mochi-mascot";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { UsageCard } from "@/components/usage/usage-card";
import { useUsageData } from "@/hooks/use-usage-data";

export const Route = createFileRoute("/")({
  component: HomePage,
});

function HomePage() {
  const { data, error, isError, isPending, isSuccess } = useUsageData();

  return (
    <main className="bg-background text-foreground min-h-screen">
      <section className="mx-auto flex min-h-screen max-w-3xl flex-col items-center justify-center gap-6 p-6">
        <MochiMascot state={isError ? "warning" : "normal"} />
        <Card className="rounded-mochi w-full max-w-md shadow-sm">
          <CardHeader className="text-center">
            <CardDescription className="font-medium tracking-[0.2em] uppercase">
              Mochi
            </CardDescription>
            <CardTitle className="text-3xl font-semibold">
              Soft alerts before hard limits.
            </CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <p className="text-muted-foreground text-center text-sm">
              Cross-platform usage companion for AI coding tools.
            </p>
            <UsageSnapshotsPanel
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

interface UsageSnapshotsPanelProps {
  data: ReturnType<typeof useUsageData>["data"];
  error: ReturnType<typeof useUsageData>["error"];
  isError: boolean;
  isPending: boolean;
  isSuccess: boolean;
}

function UsageSnapshotsPanel({
  data,
  error,
  isError,
  isPending,
  isSuccess,
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
        No provider usage snapshots yet. Connect a provider to get started.
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
