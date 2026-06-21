Homebrew casks live in the repository root at [`Casks/`](../../Casks/).

Generate or refresh them with:

```bash
node scripts/release/generate-homebrew-casks.mjs --tag v0.2.4 --cask mochi-desktop --out-dir Casks
node scripts/release/generate-homebrew-casks.mjs --tag unstable-YYYYMMDD.HHMMSS --cask mochi-unstable --out-dir Casks
```

Release workflows update these files automatically after stable and unstable publishes.
