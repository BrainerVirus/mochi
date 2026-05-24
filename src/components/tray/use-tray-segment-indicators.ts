"use client";

import { useCallback, useEffect, useLayoutEffect, useRef, type RefObject } from "react";

import {
  clearTraySegmentHover,
  clearTraySegmentPointerHover,
  createTraySegmentHoverState,
  getTraySegmentHoverTarget,
  setTraySegmentPointerHover,
  type TraySegmentHoverState,
} from "@/components/tray/tray-segment-hover-state";
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

function useHoverIndicatorHandlers(
  hoverStateRef: RefObject<TraySegmentHoverState>,
  suppressHoverEndRef: RefObject<boolean>,
  syncCurrentHover: (animate: boolean) => void,
) {
  const handleHover = useCallback(
    (tabId: string) => {
      setTraySegmentPointerHover(hoverStateRef.current, tabId);
      syncCurrentHover(true);
    },
    [hoverStateRef, syncCurrentHover],
  );

  const handleHoverEnd = useCallback(
    (tabId: string) => {
      if (!shouldHideHoverOnLeave(suppressHoverEndRef.current)) {
        return;
      }
      clearTraySegmentPointerHover(hoverStateRef.current, tabId);
      syncCurrentHover(true);
    },
    [hoverStateRef, suppressHoverEndRef, syncCurrentHover],
  );

  const handlePointerDown = useCallback(
    (tabId: string) => {
      suppressHoverEndRef.current = true;
      setTraySegmentPointerHover(hoverStateRef.current, tabId);
      syncCurrentHover(false);
    },
    [hoverStateRef, suppressHoverEndRef, syncCurrentHover],
  );

  const handlePointerUp = useCallback(() => {
    suppressHoverEndRef.current = false;
  }, [suppressHoverEndRef]);

  return { handleHover, handleHoverEnd, handlePointerDown, handlePointerUp };
}

function useHoverIndicatorSync(
  trackRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
  value: string,
) {
  const hoverStateRef = useRef(createTraySegmentHoverState());
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
        return;
      }

      const item = itemRefs.current?.get(tabId);
      if (!item) {
        return;
      }

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

  const syncCurrentHover = useCallback(
    (animate: boolean) => {
      syncHover(getTraySegmentHoverTarget(hoverStateRef.current), animate);
    },
    [syncHover],
  );

  const { handleHover, handleHoverEnd, handlePointerDown, handlePointerUp } =
    useHoverIndicatorHandlers(hoverStateRef, suppressHoverEndRef, syncCurrentHover);

  return {
    syncCurrentHover,
    hoverStateRef,
    handleHover,
    handleHoverEnd,
    handlePointerDown,
    handlePointerUp,
  };
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
    const animateActive = activeIndicator ? readIndicatorMetrics(activeIndicator).width > 0 : false;

    syncIndicators(animateActive);
  }, [tabCount, syncIndicators, value, activeIndicatorRef]);
}

function useClearHoverOnValueChange(
  value: string,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  hoverStateRef: RefObject<TraySegmentHoverState>,
) {
  const previousValueRef = useRef(value);

  useLayoutEffect(() => {
    if (previousValueRef.current === value) {
      return;
    }

    previousValueRef.current = value;
    clearTraySegmentHover(hoverStateRef.current);
    const hoverIndicator = hoverIndicatorRef.current;
    if (hoverIndicator) {
      hideHoverIndicator(hoverIndicator, false);
    }
  }, [hoverIndicatorRef, hoverStateRef, value]);
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
  syncCurrentHover: (animate: boolean) => void,
  hoverStateRef: RefObject<ReturnType<typeof createTraySegmentHoverState>>,
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

      syncCurrentHover(false);

      return true;
    },
    [activeIndicatorRef, itemRefs, syncActive, syncCurrentHover, trackRef, value],
  );

  const prepareActiveFromHover = useCallback(
    (targetTabId: string) => {
      const hoverIndicator = hoverIndicatorRef.current;
      const hoveredId = getTraySegmentHoverTarget(hoverStateRef.current);
      if (!hoverIndicator || !hoveredId) {
        return;
      }

      const handoff = mergeHoverIntoActiveStart(hoverIndicator, hoveredId, targetTabId);
      if (handoff) {
        handoffStartRef.current = handoff;
        clearTraySegmentPointerHover(hoverStateRef.current, hoveredId);
      }
    },
    [hoverIndicatorRef, hoverStateRef],
  );

  useClearHoverOnValueChange(value, hoverIndicatorRef, hoverStateRef);
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
  const {
    syncCurrentHover,
    hoverStateRef,
    handleHover,
    handleHoverEnd,
    handlePointerDown,
    handlePointerUp,
  } = useHoverIndicatorSync(trackRef, hoverIndicatorRef, itemRefs, value);

  const { prepareActiveFromHover } = useActiveIndicatorSync(
    trackRef,
    activeIndicatorRef,
    hoverIndicatorRef,
    itemRefs,
    value,
    tabCount,
    syncCurrentHover,
    hoverStateRef,
  );

  const handleSegmentValueChange = useCallback(
    (next: string, onValueChange: (value: string) => void) => {
      prepareActiveFromHover(next);
      onValueChange(next);
    },
    [prepareActiveFromHover],
  );

  return {
    syncHoverIndicator: handleHover,
    prepareActiveFromHover,
    handleHoverEnd,
    handlePointerDown,
    handlePointerUp,
    handleSegmentValueChange,
  };
}
