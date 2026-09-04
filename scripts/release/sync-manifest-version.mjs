#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";

export function setPackageVersion(src, version) {
  if (!/^(\s*"version":\s*)"[^"]*"/m.test(src)) {
    throw new Error(`package.json has no version field to set to ${version}`);
  }
  return src.replace(/^(\s*"version":\s*)"[^"]*"/m, `$1"${version}"`);
}

export function setCargoVersion(src, version) {
  return src.replace(/^version = "[^"]+"/m, `version = "${version}"`);
}

export function setCargoLockVersion(src, version) {
  if (!/\[\[package\]\]\nname = "mochi"\nversion = /.test(src)) {
    throw new Error(`Cargo.lock has no mochi package entry to set version ${version}`);
  }
  return src.replace(/(\[\[package\]\]\nname = "mochi"\nversion = )"[^"]+"/, `$1"${version}"`);
}

export function setTauriVersion(src, version) {
  if (!/^(\s*"version":\s*)"[^"]*"/m.test(src)) {
    throw new Error(`tauri.conf.json has no version field to set to ${version}`);
  }
  return src.replace(/^(\s*"version":\s*)"[^"]*"/m, `$1"${version}"`);
}

const args = process.argv.slice(2);
if (args[0] === "--set" && args[1]) {
  const version = args[1];
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    console.error(`usage: sync-manifest-version.mjs --set <MAJOR.MINOR.PATCH> (got "${version}")`);
    process.exit(2);
  }
  writeFileSync("package.json", setPackageVersion(readFileSync("package.json", "utf8"), version));
  writeFileSync(
    "src-tauri/Cargo.toml",
    setCargoVersion(readFileSync("src-tauri/Cargo.toml", "utf8"), version),
  );
  writeFileSync(
    "src-tauri/tauri.conf.json",
    setTauriVersion(readFileSync("src-tauri/tauri.conf.json", "utf8"), version),
  );
  writeFileSync(
    "src-tauri/Cargo.lock",
    setCargoLockVersion(readFileSync("src-tauri/Cargo.lock", "utf8"), version),
  );
  console.log(`manifests set to ${version}`);
}
