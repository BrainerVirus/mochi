Homebrew casks live in the repository root at [`Casks/`](../../Casks/).

Generate or refresh them with:

```bash
node scripts/release/generate-homebrew-casks.mjs --tag v0.2.4 --cask mochi-desktop --out-dir Casks
```

The release workflow regenerates the cask after a stable publish, then opens a PR into `main` and squash-merges once required checks pass (protected-branch flow).
