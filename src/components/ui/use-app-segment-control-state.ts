"use client";

import { useCallback, useRef } from "react";

import {
  useTraySegmentIndicators,
  type UseTraySegmentIndicatorsOptions,
} from "@/features/tray/components/use-tray-segment-indicators";

export function useAppSegmentControlState(
  value: string,
  itemCount: number,
  onValueChange: (value: string) => void,
  indicatorOptions: UseTraySegmentIndicatorsOptions & {
    enabled: boolean;
    contentReady?: boolean;
  },
) {
  const trackRef = useRef<HTMLDivElement>(null);
  const activeIndicatorRef = useRef<HTMLDivElement>(null);
  const hoverIndicatorRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<Map<string, HTMLButtonElement>>(new Map());

  const setItemRef = useCallback((id: string, element: HTMLButtonElement | null) => {
    if (element) {
      itemRefs.current.set(id, element);
      return;
    }
    itemRefs.current.delete(id);
  }, []);

  const { syncHoverIndicator, handleRailLeave, handleSegmentValueChange, pillReady } =
    useTraySegmentIndicators(
      trackRef,
      activeIndicatorRef,
      hoverIndicatorRef,
      value,
      indicatorOptions.enabled ? itemCount : 0,
      itemRefs,
      {
        showHover: indicatorOptions.showHover,
        contentReady: indicatorOptions.contentReady,
      },
    );

  const handleValueChange = useCallback(
    (next: string) => {
      if (next) {
        if (indicatorOptions.enabled) {
          handleSegmentValueChange(next, onValueChange);
          return;
        }
        onValueChange(next);
      }
    },
    [handleSegmentValueChange, indicatorOptions.enabled, onValueChange],
  );

  return {
    trackRef,
    activeIndicatorRef,
    hoverIndicatorRef,
    setItemRef,
    syncHoverIndicator,
    handleRailLeave,
    handleValueChange,
    pillReady,
  };
}
