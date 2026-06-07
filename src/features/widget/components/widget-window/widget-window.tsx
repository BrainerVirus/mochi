import { useRef } from "react";

import { UsageSnapshotsPanel } from "@/features/tray/components/tray-panel-content";
import { TrayPanelDivider } from "@/features/tray/components/tray-panel-divider";
import { TrayPanelFooter } from "@/features/tray/components/tray-panel-footer";
import { useTrayPanelFocusReset } from "@/features/tray/hooks/use-tray-panel-focus-reset";
import { useTrayPanelShortcuts } from "@/features/tray/hooks/use-tray-panel-shortcuts";
import { useTrayPanelState } from "@/features/tray/hooks/use-tray-panel-state";
import { quitApp } from "@/lib/tauri/commands";

export function WidgetWindow() {
  const layoutRef = useRef<HTMLDivElement>(null);
  const {
    settings,
    error,
    isError,
    isPending,
    isSuccess,
    isFetching,
    refreshAll,
    isRefreshingAll,
    refreshProviderMutation,
    selectedTab,
    refreshingProvider,
    states,
    tabs,
    handleTabChange,
    handleRefreshProvider,
  } = useTrayPanelState();

  useTrayPanelFocusReset(layoutRef);
  useTrayPanelShortcuts({
    onRefresh: () => {
      void refreshAll();
    },
    onQuit: () => {
      void quitApp();
    },
  });

  return (
    <main
      data-widget-window-shell
      className="app-window text-foreground flex h-full min-h-0 w-full flex-col overflow-hidden"
    >
      <div
        ref={layoutRef}
        className="min-h-0 flex-1 overflow-x-hidden overflow-y-auto overscroll-y-contain"
      >
        <section
          data-tray-panel-content
          className="mx-auto flex w-full max-w-[360px] min-w-0 flex-col pt-3"
        >
          <UsageSnapshotsPanel
            error={error}
            isError={isError}
            isPending={isPending}
            isSuccess={isSuccess}
            enabledProviderCount={settings?.enabled_providers.length ?? 0}
            activeTab={selectedTab}
            onTabChange={handleTabChange}
            tabs={tabs}
            states={states}
            onRefreshProvider={handleRefreshProvider}
            refreshingProvider={refreshingProvider}
          />
          <TrayPanelDivider inset data-tray-panel-separator />
          <TrayPanelFooter
            isRefreshing={isFetching || refreshProviderMutation.isPending || isRefreshingAll}
            onRefresh={() => {
              void refreshAll();
            }}
            onQuit={() => {
              void quitApp();
            }}
          />
        </section>
      </div>
    </main>
  );
}
