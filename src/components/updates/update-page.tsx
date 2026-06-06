import { getRouteApi } from "@tanstack/react-router";

import { AppWindowShell } from "@/components/layout/app-window-shell";
import { UpdatePageContent } from "@/components/updates/update-page-content";
import { useUpdateCheck } from "@/hooks/use-update-install";
import { readCachedReleaseNotes } from "@/lib/updates/release-notes-cache";

const updateRouteApi = getRouteApi("/update");

export function UpdatePage() {
  const search = updateRouteApi.useSearch();
  const { data: updateInfo, isFetching, refetch, isError, error } = useUpdateCheck();
  const cachedNotes = readCachedReleaseNotes();
  const notesSource = updateInfo?.notes ? "updater" : (cachedNotes?.source ?? "updater");
  const notesDescription =
    notesSource === "installed-release"
      ? "Release notes for the installed version."
      : "Release notes from the latest update check.";

  return (
    <AppWindowShell variant="update">
      <UpdatePageContent
        notesOnly={search.view === "notes"}
        updateAvailable={updateInfo?.available ?? false}
        version={updateInfo?.version ?? cachedNotes?.version ?? null}
        channel={updateInfo?.channel ?? cachedNotes?.channel ?? "stable"}
        notesDescription={notesDescription}
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
