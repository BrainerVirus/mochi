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

REMOTE_SHA="$(git ls-remote --heads origin "refs/heads/${BRANCH_NAME}" | awk 'NR == 1 { print $1 }')"
git checkout -B "${BRANCH_NAME}"
git commit -m "${COMMIT_MESSAGE}"
if [[ -n "${REMOTE_SHA}" ]]; then
  git push --force-with-lease="refs/heads/${BRANCH_NAME}:${REMOTE_SHA}" -u origin "${BRANCH_NAME}"
else
  git push -u origin "${BRANCH_NAME}"
fi

EXISTING_PR="$(gh pr list --head "${BRANCH_NAME}" --state open --json number --jq '.[0].number // empty')"
if [[ -n "${EXISTING_PR}" ]]; then
  PR_NUM="${EXISTING_PR}"
  echo "Reusing open PR #${PR_NUM}."
else
  CLOSED_PR="$(gh pr list \
    --head "${BRANCH_NAME}" \
    --state closed \
    --json number,mergedAt \
    --jq 'map(select(.mergedAt == null))[0].number // empty')"
  if [[ -n "${CLOSED_PR}" ]]; then
    PR_NUM="${CLOSED_PR}"
    gh pr reopen "${PR_NUM}"
    echo "Reopened PR #${PR_NUM}."
  else
    PR_URL="$(gh pr create \
      --base main \
      --head "${BRANCH_NAME}" \
      --title "${PR_TITLE}" \
      --body "Automated Homebrew cask update from the release workflow. Squash-merges after required checks pass.")"
    PR_NUM="$(gh pr view "${PR_URL}" --json number --jq .number)"
    echo "Created PR #${PR_NUM}."
  fi
fi

HEAD_SHA="$(git rev-parse HEAD)"
VALIDATION_ID="${GITHUB_RUN_ID:-manual}-${GITHUB_RUN_ATTEMPT:-1}-${HEAD_SHA}"
RUN_TITLE="Pull Request (${VALIDATION_ID})"

gh workflow run pr.yml --ref "${BRANCH_NAME}" -f "validation_id=${VALIDATION_ID}"

RUN_ID=""
ATTEMPT=0
while [[ -z "${RUN_ID}" && "${ATTEMPT}" -lt 60 ]]; do
  RUN_ID="$(gh run list \
    --workflow pr.yml \
    --branch "${BRANCH_NAME}" \
    --commit "${HEAD_SHA}" \
    --event workflow_dispatch \
    --limit 20 \
    --json databaseId,displayTitle \
    --jq ".[] | select(.displayTitle == \"${RUN_TITLE}\") | .databaseId" | head -n 1)"
  if [[ -z "${RUN_ID}" ]]; then
    sleep 2
  fi
  ATTEMPT=$((ATTEMPT + 1))
done

if [[ -z "${RUN_ID}" ]]; then
  echo "Timed out waiting for validation run ${RUN_TITLE}." >&2
  exit 1
fi

echo "Waiting for validation run ${RUN_ID}."
gh run watch "${RUN_ID}" --compact --exit-status --interval 15
gh pr merge "${PR_NUM}" --squash --delete-branch --match-head-commit "${HEAD_SHA}"
echo "Merged PR #${PR_NUM}."
