import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { queryKeys } from "@/lib/query/keys";
import { appVersion } from "@/lib/tauri/commands";

export const Route = createFileRoute("/about")({
  component: AboutPage,
});

function AboutPage() {
  const { data: version = "…" } = useQuery({
    queryKey: queryKeys.appVersion,
    queryFn: appVersion,
    staleTime: Number.POSITIVE_INFINITY,
  });

  return (
    <section className="mx-auto flex min-h-full w-full max-w-[720px] flex-col gap-6 p-6">
      <Card className="rounded-mochi shadow-sm">
        <CardHeader>
          <CardDescription className="font-medium tracking-[0.2em] uppercase">
            Mochi
          </CardDescription>
          <CardTitle className="text-3xl font-semibold">About</CardTitle>
          <CardDescription>Soft alerts before hard limits.</CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground text-sm tabular-nums">Version {version}</p>
        </CardContent>
      </Card>
    </section>
  );
}
