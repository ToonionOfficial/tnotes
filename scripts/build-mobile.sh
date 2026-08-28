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

# Resolve repo from git remote
REPO=$(gh repo view --json nameWithOwner --jq '.nameWithOwner')

echo "🚀 Triggering ${PLATFORM} ${PROFILE} build..."
echo "   Repo: ${REPO}"
echo ""

gh workflow run "${WORKFLOW}" \
  -f platform="${PLATFORM}" \
  -f profile="${PROFILE}"

echo "⏳ Waiting for run to start..."
sleep 5

# Get the latest run ID for this workflow
RUN_ID=$(gh run list \
  --workflow "${WORKFLOW}" \
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
    echo "✅ Build succeeded! Deploy it from your server:"
    echo "   ./scripts/deploy-mobile.sh ${PLATFORM} ${PROFILE}"
  } || {
    echo ""
    echo "❌ Build failed. Check the logs:"
    echo "   gh run view ${RUN_ID} --log-failed"
  }
fi
