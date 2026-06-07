import { setTrayPanelHeight, setWidgetHeight } from "@/lib/tauri/commands";
import {
  animateTrayPanelHeightTo,
  prefersReducedMotion,
  resolveTrayPanelHeight,
} from "@/lib/utils/tray-panel-height-animation";
import { TRAY_PANEL_CONTENT_SELECTOR } from "@/lib/utils/tray-panel-layout";

export function isTauriTrayPanel(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function syncTrayPanelHeightInstant(
  layout: HTMLElement,
  lastHeightRef: { current: number | null },
  target: "tray" | "widget" = "tray",
): void {
  const height = resolveTrayPanelHeight(layout);
  lastHeightRef.current = height;
  void (target === "widget" ? setWidgetHeight(height) : setTrayPanelHeight(height));
}

export function observeTrayPanelHeight(
  layout: HTMLElement,
  lastHeightRef: { current: number | null },
  isTabAnimatingRef: { current: boolean },
  target: "tray" | "widget" = "tray",
): () => void {
  let frame = 0;

  const syncHeight = () => {
    if (isTabAnimatingRef.current) {
      return;
    }

    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      syncTrayPanelHeightInstant(layout, lastHeightRef, target);
    });
  };

  const resizeObserver = new ResizeObserver(syncHeight);
  resizeObserver.observe(layout);

  const content = layout.querySelector(TRAY_PANEL_CONTENT_SELECTOR);
  if (content) {
    resizeObserver.observe(content);
  }

  syncHeight();
  window.addEventListener("resize", syncHeight);

  return () => {
    cancelAnimationFrame(frame);
    resizeObserver.disconnect();
    window.removeEventListener("resize", syncHeight);
  };
}

interface TabHeightAnimationRefs {
  lastHeightRef: { current: number | null };
  isTabAnimatingRef: { current: boolean };
  heightTweenRef: { current: ReturnType<typeof animateTrayPanelHeightTo> | null };
  isInitialTabRef: { current: boolean };
}

export function runTrayPanelTabHeightAnimation(
  layout: HTMLElement,
  refs: TabHeightAnimationRefs,
  target: "tray" | "widget" = "tray",
): () => void {
  if (refs.isInitialTabRef.current) {
    refs.isInitialTabRef.current = false;
    syncTrayPanelHeightInstant(layout, refs.lastHeightRef, target);
    return () => {};
  }

  refs.isTabAnimatingRef.current = true;
  refs.heightTweenRef.current?.kill();

  const fromHeight = refs.lastHeightRef.current ?? resolveTrayPanelHeight(layout);

  const frame = requestAnimationFrame(() => {
    const toHeight = resolveTrayPanelHeight(layout);

    if (fromHeight === toHeight || prefersReducedMotion()) {
      refs.lastHeightRef.current = toHeight;
      refs.isTabAnimatingRef.current = false;
      void (target === "widget" ? setWidgetHeight(toHeight) : setTrayPanelHeight(toHeight));
      return;
    }

    refs.heightTweenRef.current = animateTrayPanelHeightTo({
      from: fromHeight,
      to: toHeight,
      onUpdate: (height) => {
        refs.lastHeightRef.current = height;
        void (target === "widget" ? setWidgetHeight(height) : setTrayPanelHeight(height));
      },
      onComplete: () => {
        refs.lastHeightRef.current = toHeight;
        refs.isTabAnimatingRef.current = false;
        refs.heightTweenRef.current = null;
      },
    });
  });

  return () => {
    cancelAnimationFrame(frame);
    refs.heightTweenRef.current?.kill();
    refs.heightTweenRef.current = null;
    refs.isTabAnimatingRef.current = false;
  };
}
