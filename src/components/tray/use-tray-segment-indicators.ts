"use client";

import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { useCallback, useRef, type RefObject } from "react";

import {
  createHoverIndicatorQuickTo,
  hideHoverIndicator,
  observeSegmentTrackResize,
  syncActiveSegmentIndicator,
  syncHoverSegmentIndicator,
} from "@/components/tray/tray-segment-indicator";

gsap.registerPlugin(useGSAP);

function useHoverIndicatorSync(
  trackRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
  value: string,
) {
  const hoveredIdRef = useRef<string | null>(null);
  const hoverQuickToRef = useRef<{ x: gsap.QuickToFunc; width: gsap.QuickToFunc } | null>(null);

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

export function useTraySegmentIndicators(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  value: string,
  tabCount: number,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
) {
  const prevValueRef = useRef(value);
  const { syncHover, hoveredIdRef } = useHoverIndicatorSync(
    trackRef,
    hoverIndicatorRef,
    itemRefs,
    value,
  );

  const syncActive = useCallback(
    (instant: boolean) => {
      syncActiveSegmentIndicator(
        trackRef.current,
        activeIndicatorRef.current,
        itemRefs.current?.get(value),
        instant,
      );
    },
    [activeIndicatorRef, itemRefs, trackRef, value],
  );

  useGSAP(
    () => {
      const track = trackRef.current;
      if (!track) {
        return undefined;
      }

      syncActive(prevValueRef.current !== value);
      prevValueRef.current = value;

      if (hoveredIdRef.current) {
        syncHover(hoveredIdRef.current, false);
      }

      const resizeObserver = observeSegmentTrackResize(track, () => {
        syncActive(true);
        if (hoveredIdRef.current) {
          syncHover(hoveredIdRef.current, false);
        }
      });

      return () => resizeObserver.disconnect();
    },
    { dependencies: [value, tabCount, syncActive, syncHover], scope: trackRef, revertOnUpdate: false },
  );

  return { syncHoverIndicator: syncHover };
}
