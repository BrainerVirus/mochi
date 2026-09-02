#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";

export function setPackageVersion(src, version) {
  const json = JSON.parse(src);
  json.version = version;
  return JSON.stringify(json, null, 2) + "\n";
}

export function setCargoVersion(src, version) {
  return src.replace(/^version = "[^"]+"/m, `version = "${version}"`);
}

export function setTauriVersion(src, version) {
  const json = JSON.parse(src);
  json.version = version;
  return JSON.stringify(json, null, 2) + "\n";
}

const args = process.argv.slice(2);
if (args[0] === "--set" && args[1]) {
  const version = args[1];
  writeFileSync("package.json", setPackageVersion(readFileSync("package.json", "utf8"), version));
  writeFileSync(
    "src-tauri/Cargo.toml",
    setCargoVersion(readFileSync("src-tauri/Cargo.toml", "utf8"), version),
  );
  writeFileSync(
    "src-tauri/tauri.conf.json",
    setTauriVersion(readFileSync("src-tauri/tauri.conf.json", "utf8"), version),
  );
  console.log(`manifests set to ${version}`);
}
