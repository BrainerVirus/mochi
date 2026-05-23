"use client";

import { useCallback, useEffect, useLayoutEffect, useRef, type RefObject } from "react";

import {
  createHoverIndicatorQuickTo,
  hideHoverIndicator,
  mergeHoverIntoActiveStart,
  observeSegmentTrackResize,
  readIndicatorMetrics,
  shouldHideHoverOnLeave,
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
  const suppressHoverEndRef = useRef(false);

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

  const handleHoverEnd = useCallback(() => {
    if (!shouldHideHoverOnLeave(suppressHoverEndRef.current)) {
      return;
    }
    syncHover(null, true);
  }, [syncHover]);

  const handlePointerDown = useCallback((tabId: string) => {
    suppressHoverEndRef.current = true;
    hoveredIdRef.current = tabId;
  }, []);

  const handlePointerUp = useCallback(() => {
    suppressHoverEndRef.current = false;
  }, []);

  return { syncHover, hoveredIdRef, handleHoverEnd, handlePointerDown, handlePointerUp };
}

function useActiveIndicatorLayout(
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  tabCount: number,
  syncIndicators: (animateActive: boolean) => boolean,
  value: string,
) {
  useLayoutEffect(() => {
    if (tabCount === 0) {
      return;
    }

    const activeIndicator = activeIndicatorRef.current;
    const animateActive = activeIndicator
      ? readIndicatorMetrics(activeIndicator).width > 0
      : false;

    syncIndicators(animateActive);
  }, [tabCount, syncIndicators, value, activeIndicatorRef]);
}

function useSegmentTrackResize(
  trackRef: RefObject<HTMLDivElement | null>,
  syncIndicators: (animateActive: boolean) => boolean,
) {
  useEffect(() => {
    const track = trackRef.current;
    if (!track) {
      return undefined;
    }

    const resizeObserver = observeSegmentTrackResize(track, () => {
      syncIndicators(false);
    });

    return () => resizeObserver.disconnect();
  }, [syncIndicators, trackRef]);
}

function useActiveIndicatorSync(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
  value: string,
  tabCount: number,
  syncHover: (tabId: string | null, animate: boolean) => void,
  hoveredIdRef: RefObject<string | null>,
) {
  const handoffStartRef = useRef<IndicatorMetrics | null>(null);

  const syncActive = useCallback(
    (animate: boolean) => {
      syncActiveSegmentIndicator(
        trackRef.current,
        activeIndicatorRef.current,
        itemRefs.current?.get(value),
        {
          animate,
          handoffStart: handoffStartRef.current,
        },
      );
      handoffStartRef.current = null;
    },
    [activeIndicatorRef, itemRefs, trackRef, value],
  );

  const syncIndicators = useCallback(
    (animateActive: boolean) => {
      const item = itemRefs.current?.get(value);
      const activeIndicator = activeIndicatorRef.current;
      if (!trackRef.current || !activeIndicator || !item) {
        return false;
      }

      syncActive(animateActive);

      const hoveredId = hoveredIdRef.current;
      if (hoveredId) {
        syncHover(hoveredId, false);
      }

      return true;
    },
    [activeIndicatorRef, itemRefs, syncActive, syncHover, trackRef, value, hoveredIdRef],
  );

  const prepareActiveFromHover = useCallback(
    (targetTabId: string) => {
      const hoverIndicator = hoverIndicatorRef.current;
      const hoveredId = hoveredIdRef.current;
      if (!hoverIndicator || !hoveredId) {
        return;
      }

      const handoff = mergeHoverIntoActiveStart(hoverIndicator, hoveredId, targetTabId);
      if (handoff) {
        handoffStartRef.current = handoff;
        hoveredIdRef.current = null;
      }
    },
    [hoverIndicatorRef, hoveredIdRef],
  );

  useActiveIndicatorLayout(activeIndicatorRef, tabCount, syncIndicators, value);
  useSegmentTrackResize(trackRef, syncIndicators);

  return { prepareActiveFromHover };
}

export function useTraySegmentIndicators(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  value: string,
  tabCount: number,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
) {
  const { syncHover, hoveredIdRef, handleHoverEnd, handlePointerDown, handlePointerUp } =
    useHoverIndicatorSync(trackRef, hoverIndicatorRef, itemRefs, value);

  const { prepareActiveFromHover } = useActiveIndicatorSync(
    trackRef,
    activeIndicatorRef,
    hoverIndicatorRef,
    itemRefs,
    value,
    tabCount,
    syncHover,
    hoveredIdRef,
  );

  const handleSegmentValueChange = useCallback(
    (next: string, onValueChange: (value: string) => void) => {
      prepareActiveFromHover(next);
      onValueChange(next);
    },
    [prepareActiveFromHover],
  );

  return {
    syncHoverIndicator: syncHover,
    prepareActiveFromHover,
    handleHoverEnd,
    handlePointerDown,
    handlePointerUp,
    handleSegmentValueChange,
  };
}
