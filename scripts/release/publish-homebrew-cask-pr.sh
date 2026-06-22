#!/usr/bin/env bash
# ponytail: release automation opens a PR instead of pushing to protected main.
set -euo pipefail

CASK_FILE="${1:?usage: publish-homebrew-cask-pr.sh <cask-file> <commit-message> <branch> <pr-title>}"
COMMIT_MESSAGE="${2:?}"
BRANCH_NAME="${3:?}"
PR_TITLE="${4:?}"

: "${GITHUB_TOKEN:?GITHUB_TOKEN is required}"

export GH_TOKEN="${GITHUB_TOKEN}"

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"

git add "${CASK_FILE}"
if git diff --staged --quiet; then
  echo "No cask changes to commit."
  exit 0
fi

git checkout -B "${BRANCH_NAME}"
git commit -m "${COMMIT_MESSAGE}"
git push --force-with-lease -u origin "${BRANCH_NAME}"

EXISTING_PR="$(gh pr list --head "${BRANCH_NAME}" --state open --json number --jq '.[0].number // empty')"
if [[ -n "${EXISTING_PR}" ]]; then
  PR_NUM="${EXISTING_PR}"
  echo "Reusing open PR #${PR_NUM}."
else
  PR_NUM="$(gh pr create \
    --base main \
    --head "${BRANCH_NAME}" \
    --title "${PR_TITLE}" \
    --body "Automated Homebrew cask update from the release workflow. Squash-merges after required checks pass." \
    --json number \
    --jq number)"
  echo "Created PR #${PR_NUM}."
fi

if gh pr merge "${PR_NUM}" --squash --auto --delete-branch 2>/dev/null; then
  echo "Auto-merge enabled for PR #${PR_NUM}."
  exit 0
fi

echo "Auto-merge unavailable; waiting for required checks before squash merge."
gh pr checks "${PR_NUM}" --watch --interval 30
gh pr merge "${PR_NUM}" --squash --delete-branch
echo "Merged PR #${PR_NUM}."
