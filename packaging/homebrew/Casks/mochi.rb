# Homebrew cask for Mochi stable channel.
# Prefer scripts/install/install-macos-brew.sh — it resolves the latest stable release
# URL and sha256 automatically. This file documents the expected cask shape for
# a future BrainerVirus/homebrew-mochi tap.

cask "mochi" do
  version "v0.0.0"
  sha256 :no_check

  url "https://github.com/BrainerVirus/mochi/releases/download/v0.0.0/Mochi_aarch64.dmg",
      verified: "github.com/BrainerVirus/mochi/"
  name "Mochi"
  desc "Cross-platform desktop companion for AI coding tool usage"
  homepage "https://github.com/BrainerVirus/mochi"

  app "Mochi.app"

  caveats <<~EOS
    This cask template uses a pinned stable tag. For automatic latest stable release
    resolution, run:

      curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-macos-brew.sh | bash

    Add -i for the unstable channel:

      curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-macos-brew.sh | bash -s -- -i
  EOS
end
