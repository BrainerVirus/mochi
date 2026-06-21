import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { bashBin, bashSyntaxCheck } from "../install/test-helpers.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const verifySh = path.join(root, "scripts/release/verify-macos-signing-env.sh");
const importSh = path.join(root, "scripts/release/import-macos-signing-cert.sh");

function runScript(scriptPath, env = {}) {
  try {
    execFileSync(bashBin(), [scriptPath], {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, ...env },
    });
    return { ok: true, stderr: "" };
  } catch (error) {
    return {
      ok: false,
      stderr: error.stderr?.toString?.() ?? String(error),
    };
  }
}

describe("macOS signing scripts", () => {
  it("verify-macos-signing-env.sh passes bash -n", () => {
    expect(() => bashSyntaxCheck(verifySh)).not.toThrow();
  });

  it("import-macos-signing-cert.sh passes bash -n", () => {
    expect(() => bashSyntaxCheck(importSh)).not.toThrow();
  });

  it("verify fails when certificate secrets are missing", () => {
    const result = runScript(verifySh, {
      APPLE_ID: "dev@example.com",
      APPLE_PASSWORD: "app-password",
      APPLE_TEAM_ID: "TEAM123",
      KEYCHAIN_PASSWORD: "keychain-pass",
    });
    expect(result.ok).toBe(false);
    expect(result.stderr).toContain("missing APPLE_CERTIFICATE");
  });

  it("verify fails when notarization credentials are missing", () => {
    const result = runScript(verifySh, {
      APPLE_CERTIFICATE: "Zm9v",
      APPLE_CERTIFICATE_PASSWORD: "cert-pass",
      KEYCHAIN_PASSWORD: "keychain-pass",
    });
    expect(result.ok).toBe(false);
    expect(result.stderr).toContain("missing notarization credentials");
  });

  it("verify passes with Apple ID notarization credentials", () => {
    const result = runScript(verifySh, {
      APPLE_CERTIFICATE: "Zm9v",
      APPLE_CERTIFICATE_PASSWORD: "cert-pass",
      KEYCHAIN_PASSWORD: "keychain-pass",
      APPLE_ID: "dev@example.com",
      APPLE_PASSWORD: "app-password",
      APPLE_TEAM_ID: "TEAM123",
    });
    expect(result.ok).toBe(true);
  });

  it("verify passes with App Store Connect API key credentials", () => {
    const result = runScript(verifySh, {
      APPLE_CERTIFICATE: "Zm9v",
      APPLE_CERTIFICATE_PASSWORD: "cert-pass",
      KEYCHAIN_PASSWORD: "keychain-pass",
      APPLE_API_KEY: "KEYID123",
      APPLE_API_ISSUER: "issuer-uuid",
      APPLE_API_KEY_PRIVATE: "Zm9v",
    });
    expect(result.ok).toBe(true);
  });

  it("documents single-line base64 encoding for APPLE_CERTIFICATE", () => {
    const source = readFileSync(importSh, "utf8");
    expect(source).toContain("openssl base64 -d -A");
  });
});

describe(".github/workflows/release-stable.yml macOS signing", () => {
  const source = readFileSync(path.join(root, ".github/workflows/release-stable.yml"), "utf8");

  it("verifies and imports macOS signing credentials before stable builds", () => {
    expect(source).toContain("scripts/release/verify-macos-signing-env.sh");
    expect(source).toContain("scripts/release/import-macos-signing-cert.sh");
    expect(source).toContain("APPLE_CERTIFICATE");
    expect(source).toContain("KEYCHAIN_PASSWORD");
    expect(source).not.toContain("ad-hoc signing");
  });
});
