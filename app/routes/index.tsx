import { createFileRoute } from "@tanstack/react-router";

import { MochiMascot } from "@/components/mascot/mochi-mascot";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export const Route = createFileRoute("/")({
  component: HomePage,
});

function HomePage() {
  return (
    <main className="bg-background text-foreground min-h-screen">
      <section className="mx-auto flex min-h-screen max-w-3xl flex-col items-center justify-center gap-4 p-6 text-center">
        <MochiMascot state="normal" />
        <Card className="rounded-mochi w-full max-w-md shadow-sm">
          <CardHeader>
            <CardDescription className="font-medium tracking-[0.2em] uppercase">
              Mochi
            </CardDescription>
            <CardTitle className="text-3xl font-semibold">
              Soft alerts before hard limits.
            </CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <p className="text-muted-foreground text-sm">
              Cross-platform usage companion for AI coding tools.
            </p>
            <Button variant="secondary" disabled>
              Tray panel coming soon
            </Button>
          </CardContent>
        </Card>
      </section>
    </main>
  );
}
