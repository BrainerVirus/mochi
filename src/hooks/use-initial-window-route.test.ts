// @vitest-environment happy-dom
import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { navigateMock } = vi.hoisted(() => ({ navigateMock: vi.fn<() => void>() }));

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => navigateMock,
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  getCurrentWebviewWindow: () => ({ label: "settings" }),
}));

vi.mock("@/lib/tauri/commands", () => ({
  takePendingAppRoute: vi.fn<() => Promise<string | null>>(),
}));

import { takePendingAppRoute } from "@/lib/tauri/commands";

import { useInitialWindowRoute } from "./use-initial-window-route";

beforeEach(() => {
  navigateMock.mockReset();
  vi.mocked(takePendingAppRoute).mockReset();
  Object.defineProperty(window, "__TAURI_INTERNALS__", {
    value: {},
    configurable: true,
    writable: true,
  });
  window.history.replaceState(null, "", "/");
});

describe("useInitialWindowRoute pending route", () => {
  it("navigates to the pending route instead of the window default", async () => {
    vi.mocked(takePendingAppRoute).mockResolvedValue("/about");

    renderHook(() => useInitialWindowRoute());

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: "/about", replace: true });
    });
  });

  it("falls back to the window default when no route is pending", async () => {
    vi.mocked(takePendingAppRoute).mockResolvedValue(null);

    renderHook(() => useInitialWindowRoute());

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: "/settings", replace: true });
    });
  });

  it("falls back to the window default when the pending route is empty", async () => {
    vi.mocked(takePendingAppRoute).mockResolvedValue("");

    renderHook(() => useInitialWindowRoute());

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: "/settings", replace: true });
    });
  });
});
