import { createRequire } from "node:module";

import type { Plugin } from "vite";

const require = createRequire(import.meta.url);
const { defineEnv } = require("unenv") as typeof import("unenv");

type InjectMap = Record<string, string | [string, string]>;

/**
 * Nitro's legacy Rollup bundler config sets Rolldown-incompatible output options
 * and uses @rollup/plugin-inject. Normalize the nitro Vite environment for Vite 8.
 */
export function fixNitroRolldownBuild(): Plugin {
  return {
    name: "mochi:fix-nitro-rolldown-build",
    enforce: "post",
    configResolved(config) {
      const nitroEnvironment = config.environments?.nitro;
      if (!nitroEnvironment?.build) {
        return;
      }

      const rollupOptions = nitroEnvironment.build.rollupOptions as {
        output?: { generatedCode?: Record<string, unknown> };
        plugins?: Array<{ name?: string }>;
      };

      const generatedCode = rollupOptions.output?.generatedCode;
      if (generatedCode && "constBindings" in generatedCode) {
        delete generatedCode.constBindings;
        if (Object.keys(generatedCode).length === 0) {
          delete rollupOptions.output!.generatedCode;
        }
      }

      const plugins = rollupOptions.plugins;
      if (!Array.isArray(plugins)) {
        return;
      }

      const injectPluginIndex = plugins.findIndex((plugin) => plugin?.name === "inject");
      if (injectPluginIndex === -1) {
        return;
      }

      const { env } = defineEnv({
        nodeCompat: false,
        npmShims: true,
        resolve: true,
      });

      plugins.splice(injectPluginIndex, 1);

      nitroEnvironment.build.rolldownOptions ??= {};
      const rolldownOptions = nitroEnvironment.build.rolldownOptions as {
        inject?: InjectMap;
      };
      rolldownOptions.inject = env.inject as InjectMap;
    },
  };
}
