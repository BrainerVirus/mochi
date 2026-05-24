"use client";

import { useCallback, useEffect, useLayoutEffect, useRef, type RefObject } from "react";

import { isActiveIndicatorAnimating } from "@/components/tray/segment-indicator-animation";
import {
  observeSegmentTrackResize,
  readIndicatorMetrics,
  syncActiveSegmentIndicator,
} from "@/components/tray/tray-segment-indicator";

function markIndicatorPlaced(
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  indicatorPlacedRef: RefObject<boolean>,
  onPlaced?: (placed: boolean) => void,
) {
  if (!activeIndicatorRef.current) {
    onPlaced?.(false);
    return false;
  }

  const { width } = readIndicatorMetrics(activeIndicatorRef.current);
  if (width > 0) {
    indicatorPlacedRef.current = true;
    onPlaced?.(true);
    return true;
  }

  onPlaced?.(false);
  return false;
}

type PlacementTryPlace = () => boolean;

function subscribeUntilIndicatorPlaced(
  track: HTMLElement | null,
  tryPlace: PlacementTryPlace,
  indicatorPlacedRef: RefObject<boolean>,
): () => void {
  let frameId = 0;
  let attempts = 0;
  const maxAttempts = 12;

  const retryFrame = () => {
    if (tryPlace() || ++attempts >= maxAttempts) {
      return;
    }
    frameId = requestAnimationFrame(retryFrame);
  };

  frameId = requestAnimationFrame(retryFrame);

  const resizeObserver =
    track === null
      ? undefined
      : observeSegmentTrackResize(track, () => {
          if (!indicatorPlacedRef.current) {
            tryPlace();
          }
        });

  return () => {
    cancelAnimationFrame(frameId);
    resizeObserver?.disconnect();
  };
}

/** Snap the active pill once the track/items have measurable layout (settings window open, etc.). */
function useInitialIndicatorPlacement(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  tabCount: number,
  syncIndicatorsRef: RefObject<(animateActive: boolean) => boolean>,
  indicatorPlacedRef: RefObject<boolean>,
  placementTrigger: unknown,
  onPlaced: (placed: boolean) => void,
) {
  const previousTabCountRef = useRef(tabCount);
  const previousTriggerRef = useRef(placementTrigger);

  useLayoutEffect(() => {
    if (tabCount === 0) {
      indicatorPlacedRef.current = false;
      onPlaced(false);
      return undefined;
    }

    // Wait until settings content is loaded — avoid failed snaps while layout is unstable.
    if (placementTrigger === false) {
      onPlaced(false);
      return undefined;
    }

    const tabCountChanged = previousTabCountRef.current !== tabCount;
    previousTabCountRef.current = tabCount;
    const triggerBecameReady = previousTriggerRef.current === false && placementTrigger === true;
    previousTriggerRef.current = placementTrigger;

    if (tabCountChanged) {
      indicatorPlacedRef.current = false;
      onPlaced(false);
    }

    const tryPlace = () => {
      const synced = syncIndicatorsRef.current(false);
      if (!synced) {
        return false;
      }

      return markIndicatorPlaced(activeIndicatorRef, indicatorPlacedRef, onPlaced);
    };

    if (!(triggerBecameReady || !indicatorPlacedRef.current)) {
      tryPlace();
      return undefined;
    }

    return subscribeUntilIndicatorPlaced(trackRef.current, tryPlace, indicatorPlacedRef);
  }, [
    activeIndicatorRef,
    indicatorPlacedRef,
    onPlaced,
    placementTrigger,
    tabCount,
    trackRef,
    syncIndicatorsRef,
  ]);
}

function useActiveIndicatorLayout(
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  tabCount: number,
  syncIndicators: (animateActive: boolean) => boolean,
  value: string,
  indicatorPlacedRef: RefObject<boolean>,
  /** Tray page tabs: machine SELECT handles value changes; only mount + resize sync here. */
  syncOnValueChange: boolean,
  contentReady: boolean,
  onPlaced: (placed: boolean) => void,
) {
  const previousValueRef = useRef(value);

  useLayoutEffect(() => {
    if (tabCount === 0) {
      return;
    }

    const valueChanged = previousValueRef.current !== value;
    previousValueRef.current = value;

    if (!valueChanged) {
      return;
    }

    if (!syncOnValueChange) {
      return;
    }

    const animate = indicatorPlacedRef.current && contentReady;
    const synced = syncIndicators(animate);
    if (synced) {
      markIndicatorPlaced(activeIndicatorRef, indicatorPlacedRef, onPlaced);
    }
  }, [
    tabCount,
    syncIndicators,
    value,
    activeIndicatorRef,
    indicatorPlacedRef,
    syncOnValueChange,
    contentReady,
    onPlaced,
  ]);
}

function useClearHoverOnValueChange(value: string, handleRailLeave: () => void) {
  const previousValueRef = useRef(value);

  useLayoutEffect(() => {
    if (previousValueRef.current === value) {
      return;
    }

    previousValueRef.current = value;
    handleRailLeave();
  }, [handleRailLeave, value]);
}

function useSegmentTrackResize(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  syncIndicators: (animateActive: boolean) => boolean,
) {
  useEffect(() => {
    const track = trackRef.current;
    if (!track) {
      return undefined;
    }

    const resizeObserver = observeSegmentTrackResize(track, () => {
      if (isActiveIndicatorAnimating(activeIndicatorRef.current)) {
        return;
      }

      syncIndicators(false);
    });

    return () => resizeObserver.disconnect();
  }, [activeIndicatorRef, syncIndicators, trackRef]);
}

export function useActiveIndicatorSync(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
  value: string,
  tabCount: number,
  handleRailLeave: () => void,
  indicatorPlacedRef: RefObject<boolean>,
  syncOnValueChange: boolean,
  contentReady: boolean,
  onPlaced: (placed: boolean) => void,
) {
  const syncActive = useCallback(
    (animate: boolean) => {
      syncActiveSegmentIndicator(
        trackRef.current,
        activeIndicatorRef.current,
        itemRefs.current?.get(value),
        {
          animate,
        },
      );
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

      return true;
    },
    [activeIndicatorRef, itemRefs, syncActive, trackRef, value],
  );

  const syncIndicatorsRef = useRef(syncIndicators);
  syncIndicatorsRef.current = syncIndicators;

  useInitialIndicatorPlacement(
    trackRef,
    activeIndicatorRef,
    tabCount,
    syncIndicatorsRef,
    indicatorPlacedRef,
    contentReady,
    onPlaced,
  );
  useClearHoverOnValueChange(value, handleRailLeave);
  useActiveIndicatorLayout(
    activeIndicatorRef,
    tabCount,
    syncIndicators,
    value,
    indicatorPlacedRef,
    syncOnValueChange,
    contentReady,
    onPlaced,
  );
  useSegmentTrackResize(trackRef, activeIndicatorRef, syncIndicators);
}
