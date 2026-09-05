// @vitest-environment happy-dom

import { afterEach, describe, expect, it } from "vitest";

import { consumeBootHashRoute } from "./initial-window-route";

afterEach(() => {
  window.history.replaceState(null, "", "/");
});

describe("consumeBootHashRoute", () => {
  it("replaces the shell path with the hashed boot route before first paint", () => {
    window.location.hash = "#/about";
    expect(consumeBootHashRoute()).toBe("/about");
    expect(window.location.pathname).toBe("/about");
    expect(window.location.hash).toBe("");
  });

  it("leaves non-boot locations untouched", () => {
    window.history.replaceState(null, "", "/settings");
    expect(consumeBootHashRoute()).toBeNull();
    expect(window.location.pathname).toBe("/settings");
  });
});
