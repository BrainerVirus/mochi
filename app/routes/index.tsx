import { createFileRoute } from "@tanstack/react-router";

import { MochiMascot } from "@/components/mascot/mochi-mascot";

export const Route = createFileRoute("/")({
  component: HomePage,
});

function HomePage() {
  return (
    <main className="bg-mochi-cream min-h-screen text-slate-900">
      <section className="mx-auto flex min-h-screen max-w-3xl flex-col items-center justify-center gap-4 p-6 text-center">
        <MochiMascot state="normal" />
        <div className="rounded-mochi bg-white/80 px-6 py-5 shadow-sm ring-1 ring-slate-200">
          <p className="text-sm font-medium tracking-[0.2em] text-slate-500 uppercase">Mochi</p>
          <h1 className="mt-2 text-3xl font-semibold">Soft alerts before hard limits.</h1>
          <p className="mt-3 text-sm text-slate-600">
            Cross-platform usage companion for AI coding tools.
          </p>
        </div>
      </section>
    </main>
  );
}
