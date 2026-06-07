/// <reference types="vitest/config" />

import path from "node:path";
import babel from "@rolldown/plugin-babel";
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import viteReact, { reactCompilerPreset } from "@vitejs/plugin-react";
import { defineConfig } from "vite";

import { folderResolver } from "./scripts/vite-folder-resolver";

export default defineConfig({
  base: "./",
  clearScreen: false,
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
  },
  preview: {
    host: "127.0.0.1",
    port: 4173,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  resolve: {
    tsconfigPaths: true,
  },
  plugins: [
    folderResolver({ srcRoot: path.resolve(import.meta.dirname, "src") }),
    tailwindcss(),
    tanstackRouter({
      target: "react",
      routesDirectory: "app/routes",
      generatedRouteTree: "app/routeTree.gen.ts",
    }),
    viteReact(),
    babel({ presets: [reactCompilerPreset()] }),
  ],
});
