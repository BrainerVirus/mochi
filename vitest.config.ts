import path from "node:path";

import { defineConfig } from "vitest/config";

import { folderResolver } from "./scripts/vite-folder-resolver";

export default defineConfig({
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "./src"),
    },
  },
  plugins: [folderResolver({ srcRoot: path.resolve(import.meta.dirname, "./src") })],
  test: {
    include: ["src/**/*.test.ts", "scripts/**/*.test.mjs", "scripts/**/*.test.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "text-summary", "html", "lcov"],
      include: ["src/**/*.{ts,tsx}", "app/**/*.{ts,tsx}"],
      exclude: [
        "**/*.test.{ts,tsx}",
        "**/*.test-d.{ts,tsx}",
        "app/routeTree.gen.ts",
        "src/shared/components/ui/**",
        "**/*.d.ts",
        "**/types.ts",
      ],
      // thresholds: { lines: 80, functions: 80, statements: 80, branches: 75 },
    },
  },
});
