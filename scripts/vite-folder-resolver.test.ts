import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, expect, it, beforeEach, afterEach } from "vitest";

import { folderResolver } from "./vite-folder-resolver";

const writeFile = (p: string, content = ""): void => {
  fs.mkdirSync(path.dirname(p), { recursive: true });
  fs.writeFileSync(p, content);
};

describe("folderResolver", () => {
  let root: string;
  let plugin: ReturnType<typeof folderResolver>;

  beforeEach(() => {
    root = fs.mkdtempSync(path.join(os.tmpdir(), "mochi-folder-resolver-"));
    plugin = folderResolver({ srcRoot: root });
  });

  afterEach(() => {
    fs.rmSync(root, { recursive: true, force: true });
  });

  it("resolves @/X/Y/Z to src/X/Y/Z/Z.ts when the folder contains Z.ts", async () => {
    writeFile(path.join(root, "features/tray/components/tray-panel/tray-panel.ts"), "export {};");
    const result = await (plugin.resolveId as any).call(
      {},
      "@/features/tray/components/tray-panel",
      undefined,
      {},
    );
    expect(result).toBe(path.join(root, "features/tray/components/tray-panel/tray-panel.ts"));
  });

  it("prefers .tsx over .ts when both exist", async () => {
    writeFile(path.join(root, "features/tray/components/tray-panel/tray-panel.ts"));
    writeFile(path.join(root, "features/tray/components/tray-panel/tray-panel.tsx"));
    const result = await (plugin.resolveId as any).call(
      {},
      "@/features/tray/components/tray-panel",
      undefined,
      {},
    );
    expect(result).toBe(path.join(root, "features/tray/components/tray-panel/tray-panel.tsx"));
  });

  it("returns null when the source is not under the @/ alias", async () => {
    const result = await (plugin.resolveId as any).call({}, "react", undefined, {});
    expect(result).toBeNull();
  });

  it("returns null when the source is a node: import", async () => {
    const result = await (plugin.resolveId as any).call({}, "node:path", undefined, {});
    expect(result).toBeNull();
  });

  it("returns null when the source already has a file extension", async () => {
    const result = await (plugin.resolveId as any).call(
      {},
      "@/features/tray/components/tray-panel/tray-panel.tsx",
      undefined,
      {},
    );
    expect(result).toBeNull();
  });

  it("returns null when the folder does not exist", async () => {
    const result = await (plugin.resolveId as any).call(
      {},
      "@/features/tray/components/does-not-exist",
      undefined,
      {},
    );
    expect(result).toBeNull();
  });

  it("returns null when the folder exists but has no same-name file", async () => {
    fs.mkdirSync(path.join(root, "features/tray/components/empty-folder"), { recursive: true });
    const result = await (plugin.resolveId as any).call(
      {},
      "@/features/tray/components/empty-folder",
      undefined,
      {},
    );
    expect(result).toBeNull();
  });
});
