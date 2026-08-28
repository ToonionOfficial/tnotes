#!/usr/bin/env bash
# Trigger EAS local builds on GitHub Actions and optionally watch progress.
# Requires: gh (GitHub CLI) — https://cli.github.com
#
# Usage:
#   ./scripts/build-mobile.sh <ios|android|all> <development|preview|production>
#
# Examples:
#   ./scripts/build-mobile.sh android preview
#   ./scripts/build-mobile.sh ios production
#   ./scripts/build-mobile.sh all preview
set -euo pipefail

WORKFLOW="eas-build.yml"

PLATFORM="${1:?Usage: build-mobile.sh <ios|android|all> <development|preview|production>}"
PROFILE="${2:?Usage: build-mobile.sh <ios|android|all> <development|preview|production>}"

# Validate inputs
if [[ ! "$PLATFORM" =~ ^(android|ios|all)$ ]]; then
  echo "❌ Invalid platform: ${PLATFORM} (must be android, ios, or all)"
  exit 1
fi
if [[ ! "$PROFILE" =~ ^(development|preview|production)$ ]]; then
  echo "❌ Invalid profile: ${PROFILE} (must be development, preview, or production)"
  exit 1
fi

# Resolve repo from git remote and current branch
REPO=$(gh repo view --json nameWithOwner --jq '.nameWithOwner')
BRANCH=$(git branch --show-current)

# Check if current branch is pushed to remote
if ! git rev-parse --verify "origin/${BRANCH}" >/dev/null 2>&1; then
  echo "⚠️  Branch '${BRANCH}' does not exist on remote yet."
  echo "   Pushing branch to GitHub so GitHub Actions can build it..."
  git push -u origin "${BRANCH}"
  echo ""
elif git log "origin/${BRANCH}..${BRANCH}" 2>/dev/null | grep -q 'commit'; then
  echo "⚠️  You have unpushed commits on branch '${BRANCH}'."
  echo "   Pushing your commits so GitHub Actions has the latest code..."
  git push origin "${BRANCH}"
  echo ""
fi

echo "🚀 Triggering ${PLATFORM} ${PROFILE} build..."
echo "   Repo:   ${REPO}"
echo "   Branch: ${BRANCH}"
echo ""

gh workflow run "${WORKFLOW}" \
  --ref "${BRANCH}" \
  -f platform="${PLATFORM}" \
  -f profile="${PROFILE}"

echo "⏳ Waiting for run to start..."
sleep 5

# Get the latest run ID for this workflow and branch
RUN_ID=$(gh run list \
  --workflow "${WORKFLOW}" \
  --branch "${BRANCH}" \
  --limit 1 \
  --json databaseId \
  --jq '.[0].databaseId')

echo "📋 Run #${RUN_ID} started"
echo "🔗 https://github.com/${REPO}/actions/runs/${RUN_ID}"
echo ""

read -rp "Watch the build live? [Y/n] " REPLY
if [[ ! "${REPLY:-Y}" =~ ^[Nn]$ ]]; then
  gh run watch "${RUN_ID}" --exit-status && {
    echo ""
    echo "✅ Build succeeded and uploaded to Firebase App Distribution!"
    echo "   Check your email or Firebase App Distribution for the install link."
  } || {
    echo ""
    echo "❌ Build failed. Check the logs:"
    echo "   gh run view ${RUN_ID} --log-failed"
  }
fi
