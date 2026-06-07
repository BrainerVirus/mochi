"use client";

import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Field, FieldContent, FieldDescription, FieldLabel } from "@/components/ui/field";
import { Progress } from "@/components/ui/progress";
import { ReleaseNotesDialog } from "@/features/updates/components/release-notes-dialog";
import {
  useUpdateCheck,
  useUpdateInstall,
} from "@/features/updates/hooks/use-update-install/use-update-install";
import type { MochiSettings } from "@/lib/schemas/settings";
import { fetchCurrentReleaseNotes } from "@/lib/updates/current-release-notes";
import {
  readCachedReleaseNotesForChannel,
  type ReleaseNotesCache,
} from "@/lib/updates/release-notes-cache";
import {
  resolveSettingsUpdateStatusLabel,
  shouldShowSettingsInstallButton,
} from "@/lib/updates/settings-update-status";

interface SettingsUpdateSectionProps {
  channel: MochiSettings["update_channel"];
}

export function SettingsUpdateSection({ channel }: SettingsUpdateSectionProps) {
  const [notesOpen, setNotesOpen] = useState(false);
  const { data: updateInfo, isFetching, refetch, isError, error } = useUpdateCheck();
  const install = useUpdateInstall();
  const { cachedNotes, refreshNotes } = useSettingsReleaseNotes(channel);

  const updateAvailable = updateInfo?.available ?? false;
  const version = updateInfo?.version ?? cachedNotes?.version ?? null;
  const notes = updateInfo?.notes ?? cachedNotes?.notes ?? null;
  const notesSource = updateInfo?.notes ? "updater" : (cachedNotes?.source ?? "updater");
  const notesDescription =
    notesSource === "installed-release" ? "Installed version notes" : undefined;
  const statusLabel = resolveSettingsUpdateStatusLabel({
    updateAvailable,
    version,
    isFetching,
    installPhase: install.phase,
    installPending: install.isPending,
    installError: install.errorMessage,
  });
  const showInstall = shouldShowSettingsInstallButton({
    updateAvailable,
    installPending: install.isPending,
  });
  const progressPercent = computeUpdateProgressPercent(install.progress);
  const showProgress =
    install.isPending && (install.phase === "downloading" || install.phase === "installing");

  return (
    <>
      <Field className="flex-col gap-2.5 py-2.5">
        <FieldContent>
          <FieldLabel className="text-sm font-medium">Updates</FieldLabel>
          <FieldDescription className="text-[11px]">{statusLabel}</FieldDescription>
          <FieldDescription className="text-[11px]">Channel: {channel}</FieldDescription>
          {isError ? (
            <FieldDescription className="text-destructive text-[11px]">
              {error?.message ?? "Could not check for updates"}
            </FieldDescription>
          ) : null}
        </FieldContent>

        {showProgress ? (
          <UpdateProgress
            phase={install.phase === "installing" ? "installing" : "downloading"}
            progressPercent={progressPercent}
          />
        ) : null}

        <UpdateActions
          isFetching={isFetching}
          installPending={install.isPending}
          showInstall={showInstall}
          onCheck={() => {
            void refetch().then(refreshNotes);
          }}
          onInstall={() => {
            install.mutate();
          }}
          onOpenNotes={() => {
            setNotesOpen(true);
          }}
        />
      </Field>

      <ReleaseNotesDialog
        open={notesOpen}
        onOpenChange={setNotesOpen}
        version={version}
        channel={updateInfo?.channel ?? cachedNotes?.channel ?? channel}
        notesDescription={notesDescription}
        notes={notes}
        isChecking={isFetching}
        onRecheck={() => {
          void refetch().then(refreshNotes);
        }}
      />
    </>
  );
}

function useSettingsReleaseNotes(channel: MochiSettings["update_channel"]) {
  const [fetchedNotes, setFetchedNotes] = useState<ReleaseNotesCache | null>(null);
  const cachedNotes =
    fetchedNotes?.channel === channel ? fetchedNotes : readCachedReleaseNotesForChannel(channel);

  function refreshNotes() {
    void fetchCurrentReleaseNotes(channel).then((entry) => {
      if (entry) {
        setFetchedNotes(entry);
      }
    });
  }

  return { cachedNotes, refreshNotes };
}

function UpdateProgress({
  phase,
  progressPercent,
}: {
  phase: "downloading" | "installing";
  progressPercent: number;
}) {
  return (
    <div className="space-y-1.5">
      <Progress value={progressPercent} className="h-1.5" />
      <p className="text-muted-foreground text-[11px]">
        {phase === "installing" ? "Installing update…" : "Downloading update…"}
      </p>
    </div>
  );
}

function UpdateActions({
  isFetching,
  installPending,
  showInstall,
  onCheck,
  onInstall,
  onOpenNotes,
}: {
  isFetching: boolean;
  installPending: boolean;
  showInstall: boolean;
  onCheck: () => void;
  onInstall: () => void;
  onOpenNotes: () => void;
}) {
  return (
    <div className="flex flex-wrap gap-2">
      <Button type="button" size="sm" disabled={isFetching || installPending} onClick={onCheck}>
        {isFetching ? "Checking…" : "Check for updates"}
      </Button>
      {showInstall ? (
        <Button type="button" size="sm" disabled={installPending} onClick={onInstall}>
          Install update
        </Button>
      ) : null}
      <Button type="button" variant="outline" size="sm" onClick={onOpenNotes}>
        What&apos;s new
      </Button>
    </div>
  );
}

function computeUpdateProgressPercent(
  progress: { downloaded: number; total: number | null } | null,
) {
  if (!progress?.total || progress.total <= 0) {
    return progress && progress.downloaded > 0 ? 35 : 0;
  }

  return Math.min(100, Math.round((progress.downloaded / progress.total) * 100));
}
