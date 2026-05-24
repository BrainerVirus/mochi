"use client";

import { useCallback, useEffect, useLayoutEffect, useRef, type RefObject } from "react";

import {
  createHoverIndicatorQuickTo,
  executeTraySegmentIndicatorCommand,
  observeSegmentTrackResize,
  readIndicatorMetrics,
  syncActiveSegmentIndicator,
} from "@/components/tray/tray-segment-indicator";
import {
  createTraySegmentIndicatorMachine,
  transitionTraySegmentIndicator,
  type TraySegmentIndicatorCommand,
} from "@/components/tray/tray-segment-indicator-machine";

function useIndicatorCommandExecutor(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
  value: string,
) {
  const hoverQuickToRef = useRef<ReturnType<typeof createHoverIndicatorQuickTo> | null>(null);

  return useCallback(
    (commands: TraySegmentIndicatorCommand[]) => {
      const indicator = hoverIndicatorRef.current;
      if (indicator) {
        hoverQuickToRef.current ??= createHoverIndicatorQuickTo(indicator);
      }

      for (const command of commands) {
        executeTraySegmentIndicatorCommand(command, {
          track: trackRef.current,
          hoverIndicator: indicator,
          activeIndicator: activeIndicatorRef.current,
          itemRefs: itemRefs.current,
          hoverQuickTo: hoverQuickToRef.current,
          activeValue: value,
        });
      }
    },
    [activeIndicatorRef, hoverIndicatorRef, itemRefs, trackRef, value],
  );
}

function useIndicatorMachineHandlers(
  executeCommands: (commands: TraySegmentIndicatorCommand[]) => void,
) {
  const machineStateRef = useRef(createTraySegmentIndicatorMachine());

  const transition = useCallback(
    (event: Parameters<typeof transitionTraySegmentIndicator>[1]) => {
      const result = transitionTraySegmentIndicator(machineStateRef.current, event);
      machineStateRef.current = result.state;
      executeCommands(result.commands);
    },
    [executeCommands],
  );

  return {
    handleHover: (tabId: string) => transition({ type: "ITEM_ENTER", tabId }),
    handleRailLeave: () => transition({ type: "RAIL_LEAVE" }),
    handleSelect: (tabId: string) => transition({ type: "SELECT", tabId }),
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
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
  value: string,
  tabCount: number,
  handleRailLeave: () => void,
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

  useClearHoverOnValueChange(value, handleRailLeave);
  useActiveIndicatorLayout(activeIndicatorRef, tabCount, syncIndicators, value);
  useSegmentTrackResize(trackRef, syncIndicators);
}

export function useTraySegmentIndicators(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  value: string,
  tabCount: number,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
) {
  const executeCommands = useIndicatorCommandExecutor(
    trackRef,
    activeIndicatorRef,
    hoverIndicatorRef,
    itemRefs,
    value,
  );
  const { handleHover, handleRailLeave, handleSelect } =
    useIndicatorMachineHandlers(executeCommands);

  useActiveIndicatorSync(trackRef, activeIndicatorRef, itemRefs, value, tabCount, handleRailLeave);

  const handleSegmentValueChange = useCallback(
    (next: string, onValueChange: (value: string) => void) => {
      handleSelect(next);
      onValueChange(next);
    },
    [handleSelect],
  );

  return {
    syncHoverIndicator: handleHover,
    handleRailLeave,
    handleSegmentValueChange,
  };
}
