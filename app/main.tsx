import { RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { getRouter } from "./router";

// oxlint-disable-next-line import/no-unassigned-import -- Vite injects the global stylesheet.
import "@/styles/index.css";
import { consumeBootHashRoute } from "@/lib/tauri/initial-window-route";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("Missing #root element for Mochi.");
}

// Fresh Tauri windows boot at `index.html#/path`; apply the route
// synchronously so first paint renders it (not the tray panel at `/`).
consumeBootHashRoute();

createRoot(rootElement).render(
  <StrictMode>
    <RouterProvider router={getRouter()} />
  </StrictMode>,
);
