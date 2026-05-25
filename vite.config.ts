/// <reference types="vitest/config" />

import tailwindcss from "@tailwindcss/vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite";

import { fixNitroRolldownBuild } from "./vite.fix-nitro-rolldown";
import { writeTauriSpaShell } from "./vite.write-tauri-shell";

export default defineConfig({
  clearScreen: false,
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
  },
  preview: {
    host: "127.0.0.1",
  },
  envPrefix: ["VITE_", "TAURI_"],
  resolve: {
    tsconfigPaths: true,
  },
  plugins: [
    tailwindcss(),
    tanstackStart({
      srcDirectory: "app",
    }),
    viteReact(),
    nitro(),
    fixNitroRolldownBuild(),
    writeTauriSpaShell(),
  ],
});
