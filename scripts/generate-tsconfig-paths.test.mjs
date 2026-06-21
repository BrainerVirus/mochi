import { execFileSync } from "node:child_process";
import path from "node:path";

import { describe, expect, it } from "vitest";

const script = path.join(import.meta.dirname, "generate-tsconfig-paths.mjs");

describe("generate-tsconfig-paths.mjs", () => {
  it("emits folder-resolver aliases for known src areas", () => {
    const output = execFileSync(process.execPath, [script], {
      cwd: path.resolve(import.meta.dirname, ".."),
      encoding: "utf8",
    });
    const paths = JSON.parse(output);

    expect(paths["@/lib/utils/format-reset-line"]).toBeDefined();
    expect(paths["@/components/mascot/mochi-chibi"]).toBeDefined();
    expect(Object.keys(paths).some((key) => key.startsWith("@/features/tray/components/"))).toBe(
      true,
    );
  });
});
