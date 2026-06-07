"use client";

import { useCallback, useEffect, useRef, useState, type RefObject } from "react";

import {
  createHoverIndicatorQuickTo,
  executeTraySegmentIndicatorCommand,
  releaseSegmentIndicators,
} from "@/features/tray/components/tray-segment-indicator";
import {
  createTraySegmentIndicatorMachine,
  transitionTraySegmentIndicator,
  type TraySegmentIndicatorCommand,
} from "@/features/tray/components/tray-segment-indicator-machine";
import { useActiveIndicatorSync } from "@/features/tray/components/use-tray-segment-indicator-sync";

function useIndicatorCleanup(
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  hoverQuickToRef: RefObject<ReturnType<typeof createHoverIndicatorQuickTo> | null>,
) {
  const indicatorsRef = useRef({
    active: null as HTMLDivElement | null,
    hover: null as HTMLDivElement | null,
  });
  indicatorsRef.current = {
    active: activeIndicatorRef.current,
    hover: hoverIndicatorRef.current,
  };

  useEffect(() => {
    return () => {
      hoverQuickToRef.current = null;
      releaseSegmentIndicators(indicatorsRef.current.active, indicatorsRef.current.hover);
    };
  }, [hoverQuickToRef]);
}

function useIndicatorCommandExecutor(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
  value: string,
  hoverQuickToRef: RefObject<ReturnType<typeof createHoverIndicatorQuickTo> | null>,
) {
  return useCallback(
    (commands: TraySegmentIndicatorCommand[]) => {
      const indicator = hoverIndicatorRef.current;

      for (const command of commands) {
        if (command.type === "moveHover" && indicator) {
          hoverQuickToRef.current ??= createHoverIndicatorQuickTo(indicator);
        }

        executeTraySegmentIndicatorCommand(command, {
          track: trackRef.current,
          hoverIndicator: indicator,
          activeIndicator: activeIndicatorRef.current,
          itemRefs: itemRefs.current,
          hoverQuickTo: hoverQuickToRef.current,
          activeValue: value,
          resetHoverQuickTo: () => {
            hoverQuickToRef.current = null;
          },
        });
      }
    },
    [activeIndicatorRef, hoverIndicatorRef, hoverQuickToRef, itemRefs, trackRef, value],
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

export interface UseTraySegmentIndicatorsOptions {
  /** When false, only the active pill animates — no cursor-following hover pill. */
  showHover?: boolean;
  /** When false, snap on tab change and re-place when it becomes true (settings load). Default true. */
  contentReady?: boolean;
}

/**
 * Tray page tabs animate via machine SELECT (moveActive) for hover handoff; inline
 * and settings page tabs defer to the value layout effect so metrics are read after
 * DOM commit and never double-tween on click.
 */
export function shouldRunMachineSelectOnValueChange(
  options: Pick<UseTraySegmentIndicatorsOptions, "showHover">,
): boolean {
  return options.showHover ?? true;
}

export function shouldSyncActiveOnValueChange(
  options: Pick<UseTraySegmentIndicatorsOptions, "showHover">,
): boolean {
  return !shouldRunMachineSelectOnValueChange(options);
}

export function shouldAnimateActiveOnValueChange(
  indicatorPlaced: boolean,
  contentReady: boolean,
): boolean {
  return indicatorPlaced && contentReady;
}

export function useTraySegmentIndicators(
  trackRef: RefObject<HTMLDivElement | null>,
  activeIndicatorRef: RefObject<HTMLDivElement | null>,
  hoverIndicatorRef: RefObject<HTMLDivElement | null>,
  value: string,
  tabCount: number,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
  options: UseTraySegmentIndicatorsOptions = {},
) {
  const showHover = options.showHover ?? true;
  const contentReady = options.contentReady ?? true;
  const syncOnValueChange = shouldSyncActiveOnValueChange({ showHover });
  const hoverQuickToRef = useRef<ReturnType<typeof createHoverIndicatorQuickTo> | null>(null);
  const indicatorPlacedRef = useRef(false);
  const [pillReady, setPillReady] = useState(false);
  const handlePlaced = useCallback((placed: boolean) => {
    setPillReady(placed);
  }, []);

  useIndicatorCleanup(activeIndicatorRef, hoverIndicatorRef, hoverQuickToRef);

  const executeCommands = useIndicatorCommandExecutor(
    trackRef,
    activeIndicatorRef,
    hoverIndicatorRef,
    itemRefs,
    value,
    hoverQuickToRef,
  );
  const { handleHover, handleRailLeave, handleSelect } =
    useIndicatorMachineHandlers(executeCommands);

  useActiveIndicatorSync(
    trackRef,
    activeIndicatorRef,
    itemRefs,
    value,
    tabCount,
    handleRailLeave,
    indicatorPlacedRef,
    syncOnValueChange,
    contentReady,
    handlePlaced,
  );

  const handleSegmentValueChange = useCallback(
    (next: string, onValueChange: (value: string) => void) => {
      if (shouldRunMachineSelectOnValueChange({ showHover })) {
        handleSelect(next);
      }
      onValueChange(next);
    },
    [handleSelect, showHover],
  );

  return {
    syncHoverIndicator: showHover ? handleHover : () => {},
    handleRailLeave: showHover ? handleRailLeave : () => {},
    handleSegmentValueChange,
    pillReady,
  };
}
