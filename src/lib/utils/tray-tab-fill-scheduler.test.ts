import { describe, expect, it } from "vitest";

import {
  isTrayTabFillPending,
  markTrayTabFillPending,
  markTrayTabFillReady,
  resetTrayTabFillSchedulerForTests,
  runWhenTrayTabFillReady,
} from "./tray-tab-fill-scheduler";

describe("trayTabFillScheduler", () => {
  it("runs fill callbacks immediately when no tab morph is pending", () => {
    resetTrayTabFillSchedulerForTests();

    let ran = false;
    const cancel = runWhenTrayTabFillReady(() => {
      ran = true;
    });

    expect(ran).toBe(true);
    cancel();
  });

  it("defers fill callbacks until the tray tab morph completes", () => {
    resetTrayTabFillSchedulerForTests();
    markTrayTabFillPending();

    const runs: string[] = [];
    runWhenTrayTabFillReady(() => {
      runs.push("first");
    });
    runWhenTrayTabFillReady(() => {
      runs.push("second");
    });

    expect(runs).toEqual([]);
    expect(isTrayTabFillPending()).toBe(true);

    markTrayTabFillReady();

    expect(runs).toEqual(["first", "second"]);
    expect(isTrayTabFillPending()).toBe(false);
  });

  it("cancels deferred callbacks when the meter unmounts before morph completes", () => {
    resetTrayTabFillSchedulerForTests();
    markTrayTabFillPending();

    let ran = false;
    const cancel = runWhenTrayTabFillReady(() => {
      ran = true;
    });

    cancel();
    markTrayTabFillReady();

    expect(ran).toBe(false);
  });

  it("does not let stale ready signals release a newer tab fill", () => {
    resetTrayTabFillSchedulerForTests();
    const staleToken = markTrayTabFillPending();
    const currentToken = markTrayTabFillPending();
    const runs: string[] = [];

    markTrayTabFillReady(staleToken);
    runWhenTrayTabFillReady(() => {
      runs.push("fill");
    });

    expect(runs).toEqual([]);
    expect(isTrayTabFillPending()).toBe(true);

    markTrayTabFillReady(currentToken);

    expect(runs).toEqual(["fill"]);
    expect(isTrayTabFillPending()).toBe(false);
  });
});
