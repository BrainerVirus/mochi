import { useMemo } from "react";

import { MochiChibi } from "@/components/mascot/mochi-chibi";
import { ScrollFadeRegion } from "@/components/tray/scroll-fade-region";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";
import { useUpdateInstall } from "@/hooks/use-update-install";
import { splitPatchNotesSections, type PatchNotesSection } from "@/lib/updates/format-patch-notes";
import { trayPanelSpacing } from "@/lib/utils/tray-panel-spacing";

interface UpdatePageContentProps {
  notesOnly: boolean;
  updateAvailable: boolean;
  version: string | null;
  channel: string;
  notes: string | null;
  isChecking: boolean;
  checkError: string | null;
  onRecheck: () => void;
}

export function UpdatePageContent(props: UpdatePageContentProps) {
  const sections = useMemo(() => splitPatchNotesSections(props.notes), [props.notes]);
  const install = useUpdateInstall();

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <UpdatePageHeader {...props} sections={sections} installPhase={install.phase} />
      <Separator />
      <UpdateNotesBody {...props} sections={sections} />
      <UpdatePageFooter {...props} install={install} />
    </div>
  );
}

function UpdatePageHeader({
  notesOnly,
  updateAvailable,
  version,
  channel,
  sections,
  installPhase,
}: Pick<UpdatePageContentProps, "notesOnly" | "updateAvailable" | "version" | "channel"> & {
  sections: PatchNotesSection[];
  installPhase: "idle" | "downloading" | "installing" | "error";
}) {
  const showChibi = !notesOnly && installPhase === "idle" && sections.length <= 2;
  const title = notesOnly
    ? "What's new"
    : updateAvailable
      ? `Update to ${version ?? "latest"}`
      : "You're up to date";

  return (
    <header className={`shrink-0 ${trayPanelSpacing.contentX} pt-4 pb-3`}>
      <div className="flex items-start gap-3">
        {showChibi ? <MochiChibi className="size-12 shrink-0" /> : null}
        <div className="min-w-0 flex-1">
          <h1 className="text-base font-semibold tracking-tight">{title}</h1>
          <p className="text-muted-foreground mt-1 text-xs">
            {notesOnly ? "Release notes from your last update check." : `Channel: ${channel}`}
          </p>
        </div>
      </div>
    </header>
  );
}

function UpdateNotesBody({
  notesOnly,
  updateAvailable,
  isChecking,
  checkError,
  sections,
}: Pick<UpdatePageContentProps, "notesOnly" | "updateAvailable" | "isChecking" | "checkError"> & {
  sections: PatchNotesSection[];
}) {
  return (
    <ScrollFadeRegion
      orientation="vertical"
      className="min-h-0 flex-1"
      scrollClassName="overscroll-y-contain"
    >
      <div className={`${trayPanelSpacing.contentX} py-3`}>
        {checkError ? <p className="text-destructive text-xs">{checkError}</p> : null}
        {sections.length > 0 ? (
          <PatchNotesSections sections={sections} />
        ) : (
          <p className="text-muted-foreground text-xs leading-relaxed">
            {resolveEmptyNotesMessage({ notesOnly, isChecking, updateAvailable })}
          </p>
        )}
      </div>
    </ScrollFadeRegion>
  );
}

function PatchNotesSections({ sections }: { sections: PatchNotesSection[] }) {
  return (
    <div className="flex flex-col gap-4">
      {sections.map((section) => (
        <section key={section.title ?? section.items.join("-")}>
          {section.title ? (
            <h2 className="text-foreground mb-1.5 text-xs font-semibold">{section.title}</h2>
          ) : null}
          <ul className="text-muted-foreground list-disc space-y-1 pl-4 text-xs leading-relaxed">
            {section.items.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

function UpdatePageFooter({
  notesOnly,
  updateAvailable,
  isChecking,
  onRecheck,
  install,
}: Pick<UpdatePageContentProps, "notesOnly" | "updateAvailable" | "isChecking" | "onRecheck"> & {
  install: ReturnType<typeof useUpdateInstall>;
}) {
  if (notesOnly) {
    return (
      <footer
        className={`shrink-0 ${trayPanelSpacing.contentX} ${trayPanelSpacing.footerBottom} py-2`}
      >
        <Button
          type="button"
          variant="outline"
          className="w-full"
          disabled={isChecking}
          onClick={onRecheck}
        >
          {isChecking ? "Checking…" : "Check for updates"}
        </Button>
      </footer>
    );
  }

  const progressPercent = computeProgressPercent(install.progress);
  const showProgress =
    install.phase === "downloading" || install.phase === "installing" || install.isPending;

  return (
    <footer
      className={`shrink-0 ${trayPanelSpacing.contentX} ${trayPanelSpacing.footerBottom} pt-2`}
    >
      {showProgress ? (
        <div className="space-y-2 pb-2">
          <Progress value={progressPercent} className="h-1.5" />
          <p className="text-muted-foreground text-center text-[11px]">
            {install.phase === "installing" ? "Installing update…" : "Downloading update…"}
          </p>
        </div>
      ) : null}
      {install.errorMessage ? (
        <p className="text-destructive pb-2 text-center text-[11px]">{install.errorMessage}</p>
      ) : null}
      <div className="flex gap-2">
        <Button
          type="button"
          variant="outline"
          className="flex-1"
          disabled={isChecking || install.isPending}
          onClick={onRecheck}
        >
          {isChecking ? "Checking…" : "Check again"}
        </Button>
        {updateAvailable ? (
          <Button
            type="button"
            className="flex-1"
            disabled={install.isPending}
            onClick={() => {
              install.mutate();
            }}
          >
            {install.isPending ? "Updating…" : "Install update"}
          </Button>
        ) : null}
      </div>
    </footer>
  );
}

function resolveEmptyNotesMessage({
  notesOnly,
  isChecking,
  updateAvailable,
}: Pick<UpdatePageContentProps, "notesOnly" | "isChecking" | "updateAvailable">) {
  if (isChecking) {
    return "Checking for updates…";
  }

  if (notesOnly) {
    return "No release notes cached yet. Check for updates to fetch the latest notes.";
  }

  if (updateAvailable) {
    return "Release notes will appear here when the updater manifest includes them.";
  }

  return "Mochi is up to date.";
}

function computeProgressPercent(progress: { downloaded: number; total: number | null } | null) {
  if (!progress?.total || progress.total <= 0) {
    return progress && progress.downloaded > 0 ? 35 : 0;
  }

  return Math.min(100, Math.round((progress.downloaded / progress.total) * 100));
}
