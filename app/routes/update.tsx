import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";

import { AppWindowShell } from "@/components/layout/app-window-shell";
import { UpdatePageContent } from "@/components/updates/update-page-content";
import { useUpdateCheck } from "@/hooks/use-update-install";
import { readCachedReleaseNotes } from "@/lib/updates/release-notes-cache";

const updateSearchSchema = z.object({
  view: z.enum(["notes"]).optional(),
});

export const Route = createFileRoute("/update")({
  ssr: false,
  validateSearch: updateSearchSchema,
  component: UpdateRoutePage,
});

function UpdateRoutePage() {
  const search = Route.useSearch();
  const { data: updateInfo, isFetching, refetch, isError, error } = useUpdateCheck();
  const cachedNotes = readCachedReleaseNotes();

  return (
    <AppWindowShell variant="update">
      <UpdatePageContent
        notesOnly={search.view === "notes"}
        updateAvailable={updateInfo?.available ?? false}
        version={updateInfo?.version ?? cachedNotes?.version ?? null}
        channel={updateInfo?.channel ?? cachedNotes?.channel ?? "stable"}
        notes={updateInfo?.notes ?? cachedNotes?.notes ?? null}
        isChecking={isFetching}
        checkError={isError ? (error?.message ?? "Could not check for updates") : null}
        onRecheck={() => {
          void refetch();
        }}
      />
    </AppWindowShell>
  );
}
