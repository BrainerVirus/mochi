# Homebrew cask for Mochi unstable channel.
# Prefer scripts/install/install-macos-brew.sh — pass -i to resolve the latest prerelease
# URL and sha256 automatically. This file documents the expected cask shape for
# a future BrainerVirus/homebrew-mochi tap.

cask "mochi-unstable" do
  version "unstable"
  sha256 :no_check

  url "https://github.com/BrainerVirus/mochi/releases/download/unstable/Mochi_unstable_aarch64.dmg",
      verified: "github.com/BrainerVirus/mochi/"
  name "Mochi (Unstable)"
  desc "Cross-platform desktop companion for AI coding tool usage (unstable channel)"
  homepage "https://github.com/BrainerVirus/mochi"

  app "Mochi.app"

  caveats <<~EOS
    This cask template uses a rolling unstable tag. For automatic latest prerelease
    resolution, run:

      curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-macos-brew.sh | bash -s -- -i
  EOS
end
