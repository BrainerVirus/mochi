# Contributing

This repository uses [GitHub Flow](https://docs.github.com/en/get-started/using-github/github-flow).

## Branching

- **`main`** is the default branch and should stay deployable.
- Do all work on **short-lived feature branches** created from `main` (for example `feat/design-system`, `fix/tray-icon`).
- Open a **pull request** to merge into `main`; avoid long-lived integration branches such as `develop`.

## Typical workflow

```bash
git checkout main
git pull origin main
git checkout -b feat/your-change
# commit, push, open PR
```

## Before you open a PR

Follow [AGENTS.md](../AGENTS.md) and [docs/tech-stack.md](tech-stack.md) for stack conventions and verification commands.
