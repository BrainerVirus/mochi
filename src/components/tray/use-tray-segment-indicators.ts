"use client";

import { useCallback, useEffect, useLayoutEffect, useRef, type RefObject } from "react";

import {
  createHoverIndicatorQuickTo,
  hideHoverIndicator,
  observeSegmentTrackResize,
  syncActiveSegmentIndicator,
  syncHoverSegmentIndicator,
  type IndicatorMetrics,
} from "@/components/tray/tray-segment-indicator";

function useHoverIndicatorSync(
  trackRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
  value: string,
) {
  const hoveredIdRef = useRef<string | null>(null);
  const hoverQuickToRef = useRef<ReturnType<typeof createHoverIndicatorQuickTo> | null>(null);

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

function useActiveIndicatorLayout(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
  value: string,
  tabCount: number,
  hasPreviousMetrics: boolean,
  syncActive: (animate: boolean) => void,
  syncHover: (tabId: string | null, animate: boolean) => void,
  hoveredIdRef: RefObject<string | null>,
) {
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

      syncActive(hasPreviousMetrics);

      if (hoveredIdRef.current) {
        syncHover(hoveredIdRef.current, false);
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
    hasPreviousMetrics,
    syncActive,
    syncHover,
    trackRef,
    activeIndicatorRef,
    itemRefs,
    hoveredIdRef,
  ]);
}

function useSegmentTrackResize(
  trackRef: RefObject<HTMLDivElement | null>,
  syncActive: (animate: boolean) => void,
  syncHover: (tabId: string | null, animate: boolean) => void,
  hoveredIdRef: RefObject<string | null>,
) {
  useEffect(() => {
    const track = trackRef.current;
    if (!track) return undefined;

    const resizeObserver = observeSegmentTrackResize(track, () => {
      syncActive(false);
      const hoveredId = hoveredIdRef.current;
      if (hoveredId) syncHover(hoveredId, false);
    });

    return () => resizeObserver.disconnect();
  }, [syncActive, syncHover, hoveredIdRef, trackRef]);
}

export function useTraySegmentIndicators(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  value: string,
  tabCount: number,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
) {
  const prevActiveMetricsRef = useRef<IndicatorMetrics | null>(null);
  const { syncHover, hoveredIdRef } = useHoverIndicatorSync(
    trackRef,
    hoverIndicatorRef,
    itemRefs,
    value,
  );

  const syncActive = useCallback(
    (animate: boolean) => {
      const metrics = syncActiveSegmentIndicator(
        trackRef.current,
        activeIndicatorRef.current,
        itemRefs.current?.get(value),
        prevActiveMetricsRef.current,
        animate,
      );

      if (metrics) {
        prevActiveMetricsRef.current = metrics;
      }
    },
    [activeIndicatorRef, itemRefs, trackRef, value],
  );

  useActiveIndicatorLayout(
    trackRef,
    activeIndicatorRef,
    itemRefs,
    value,
    tabCount,
    prevActiveMetricsRef.current !== null,
    syncActive,
    syncHover,
    hoveredIdRef,
  );

  useSegmentTrackResize(trackRef, syncActive, syncHover, hoveredIdRef);

  return { syncHoverIndicator: syncHover };
}
