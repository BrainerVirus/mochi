"use client";

import { useCallback, useEffect, useLayoutEffect, useRef, type RefObject } from "react";

import {
  createActiveIndicatorQuickTo,
  createHoverIndicatorQuickTo,
  hideHoverIndicator,
  observeSegmentTrackResize,
  syncActiveSegmentIndicator,
  syncHoverSegmentIndicator,
} from "@/components/tray/tray-segment-indicator";

type IndicatorQuickTo = ReturnType<typeof createActiveIndicatorQuickTo>;

function useHoverIndicatorSync(
  trackRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
  value: string,
) {
  const hoveredIdRef = useRef<string | null>(null);
  const hoverQuickToRef = useRef<IndicatorQuickTo | null>(null);

  const syncHover = useCallback(
    (tabId: string | null, animate: boolean) => {
      const indicator = hoverIndicatorRef.current;
      if (!indicator) {
        return;
      }

      if (!tabId) {
        hideHoverIndicator(indicator, animate);
        hoveredIdRef.current = null;
        return;
      }

      const item = itemRefs.current?.get(tabId);
      if (!item) {
        return;
      }

      hoveredIdRef.current = tabId;
      hoverQuickToRef.current ??= createHoverIndicatorQuickTo(indicator);

      syncHoverSegmentIndicator(
        trackRef.current,
        indicator,
        item,
        hoverQuickToRef.current,
        value,
        tabId,
      );
    },
    [hoverIndicatorRef, itemRefs, trackRef, value],
  );

  return { syncHover, hoveredIdRef };
}

function useActiveIndicatorSync(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
  value: string,
  tabCount: number,
  hoveredIdRef: RefObject<string | null>,
  syncHover: (tabId: string | null, animate: boolean) => void,
) {
  const hasPositionedActiveRef = useRef(false);
  const activeQuickToRef = useRef<IndicatorQuickTo | null>(null);

  const syncActive = useCallback(
    (animate: boolean) => {
      const indicator = activeIndicatorRef.current;
      if (!indicator) {
        return;
      }

      activeQuickToRef.current ??= createActiveIndicatorQuickTo(indicator);

      syncActiveSegmentIndicator(
        trackRef.current,
        indicator,
        itemRefs.current?.get(value),
        animate,
        activeQuickToRef.current,
      );
    },
    [activeIndicatorRef, itemRefs, trackRef, value],
  );

  useLayoutEffect(() => {
    let cancelled = false;
    let frameId = 0;

    const run = () => {
      if (cancelled) {
        return;
      }

      const item = itemRefs.current?.get(value);
      if (!trackRef.current || !activeIndicatorRef.current || !item) {
        frameId = requestAnimationFrame(run);
        return;
      }

      syncActive(hasPositionedActiveRef.current);
      hasPositionedActiveRef.current = true;

      const hoveredId = hoveredIdRef.current;
      if (hoveredId) {
        syncHover(hoveredId, false);
      }
    };

    run();

    return () => {
      cancelled = true;
      cancelAnimationFrame(frameId);
    };
  }, [
    value,
    tabCount,
    syncActive,
    syncHover,
    trackRef,
    activeIndicatorRef,
    itemRefs,
    hoveredIdRef,
  ]);

  return syncActive;
}

export function useTraySegmentIndicators(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  value: string,
  tabCount: number,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
) {
  const { syncHover, hoveredIdRef } = useHoverIndicatorSync(
    trackRef,
    hoverIndicatorRef,
    itemRefs,
    value,
  );

  const syncActive = useActiveIndicatorSync(
    trackRef,
    activeIndicatorRef,
    itemRefs,
    value,
    tabCount,
    hoveredIdRef,
    syncHover,
  );

  useEffect(() => {
    const track = trackRef.current;
    if (!track) {
      return undefined;
    }

    const resizeObserver = observeSegmentTrackResize(track, () => {
      syncActive(false);
      const hoveredId = hoveredIdRef.current;
      if (hoveredId) {
        syncHover(hoveredId, false);
      }
    });

    return () => resizeObserver.disconnect();
  }, [syncActive, syncHover, hoveredIdRef, trackRef]);

  return { syncHoverIndicator: syncHover };
}
