import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";

import { settingsQueryOptions } from "@/lib/query/settings";
import { createUpdateCheckQueryOptions } from "@/lib/query/update-check";
import { UpdateDownloadProgressSchema, type UpdateDownloadProgress } from "@/lib/schemas/usage";
import { checkForUpdate, installUpdate } from "@/lib/tauri/commands";
import { isTauriRuntime } from "@/lib/tauri/runtime";
import { markPostUpdateRefreshPending } from "@/lib/updates/release-notes-cache";

export function useUpdateCheck(enabled = isTauriRuntime()) {
  const { data: settings } = useQuery(settingsQueryOptions);
  const channel = settings?.update_channel ?? "stable";

  return useQuery({
    ...createUpdateCheckQueryOptions(channel),
    enabled: enabled && Boolean(settings),
  });
}

export function useUpdateInstall() {
  const queryClient = useQueryClient();
  const { data: settings } = useQuery(settingsQueryOptions);
  const channel = settings?.update_channel ?? "stable";
  const [progress, setProgress] = useState<UpdateDownloadProgress | null>(null);
  const [phase, setPhase] = useState<"idle" | "downloading" | "installing" | "error">("idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return undefined;
    }

    const unlistenPromises = [
      listen<unknown>("update-download-progress", (event) => {
        const parsed = UpdateDownloadProgressSchema.safeParse(event.payload);
        if (parsed.success) {
          setProgress(parsed.data);
          setPhase("downloading");
        }
      }),
      listen("update-install-started", () => {
        setPhase("installing");
      }),
    ];

    return () => {
      void Promise.all(unlistenPromises).then((unlisteners) => {
        for (const unlisten of unlisteners) {
          unlisten();
        }
      });
    };
  }, []);

  const mutation = useMutation({
    mutationFn: async () => {
      setPhase("downloading");
      setProgress({ downloaded: 0, total: null });
      setErrorMessage(null);
      markPostUpdateRefreshPending();
      await installUpdate(channel);
    },
    onError: (error) => {
      setPhase("error");
      setErrorMessage(error instanceof Error ? error.message : "Update failed");
    },
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: ["update", "check"] });
    },
  });

  const checkNow = useCallback(async (nextChannel: string) => {
    return checkForUpdate(nextChannel);
  }, []);

  return {
    ...mutation,
    progress,
    phase: mutation.isPending ? phase : phase === "error" ? "error" : "idle",
    errorMessage,
    checkNow,
  };
}
