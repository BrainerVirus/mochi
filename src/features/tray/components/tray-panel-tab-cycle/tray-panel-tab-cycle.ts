import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import {
  cycleHorizontalScrollBackward,
  cycleHorizontalScrollForward,
} from "@/features/tray/components/scroll-fade-cycle";

const TRAY_TAB_SELECTOR = "[data-tray-tab-id]";

function scrollTrayTabIntoView(scrollEl: HTMLDivElement, tabId: string) {
  const trigger = scrollEl.querySelector<HTMLElement>(
    `${TRAY_TAB_SELECTOR}[data-tray-tab-id="${tabId}"]`,
  );
  trigger?.scrollIntoView({ behavior: "smooth", block: "nearest", inline: "nearest" });
}

export function cycleTrayPanelTabs(
  scrollEl: HTMLDivElement,
  tabs: TrayPanelTab[],
  currentValue: string,
  direction: "forward" | "backward",
  onValueChange: (value: string) => void,
  fadeInset: number,
) {
  const currentIndex = tabs.findIndex((tab) => tab.id === currentValue);
  if (currentIndex === -1) {
    return;
  }

  const nextIndex =
    direction === "forward"
      ? Math.min(currentIndex + 1, tabs.length - 1)
      : Math.max(currentIndex - 1, 0);

  if (nextIndex === currentIndex) {
    if (direction === "forward") {
      cycleHorizontalScrollForward(scrollEl, fadeInset);
    } else {
      cycleHorizontalScrollBackward(scrollEl, fadeInset);
    }
    return;
  }

  const nextTab = tabs[nextIndex];
  onValueChange(nextTab.id);

  requestAnimationFrame(() => {
    scrollTrayTabIntoView(scrollEl, nextTab.id);
  });
}
