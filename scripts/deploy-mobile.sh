#!/usr/bin/env bash
# Download a mobile build from GitHub Actions and serve it for OTA install.
# Requires: gh (GitHub CLI), qrencode (for terminal QR codes)
#
# Usage:
#   ./scripts/deploy-mobile.sh <ios|android> [profile]
#
# Examples:
#   ./scripts/deploy-mobile.sh ios preview
#   ./scripts/deploy-mobile.sh android production
#
# Configuration (env vars):
#   BUILDS_DOMAIN  — your server's domain (e.g. builds.toonion.net)
#   BUILDS_DIR     — directory to serve files from (default: /srv/mobile-builds)
set -euo pipefail

PLATFORM="${1:?Usage: deploy-mobile.sh <ios|android> <development|preview|production>}"
PROFILE="${2:?Usage: deploy-mobile.sh <ios|android> <development|preview|production>}"

BUILDS_DOMAIN="${BUILDS_DOMAIN:?Set BUILDS_DOMAIN to your server's domain (e.g. builds.toonion.net)}"
BUILDS_DIR="${BUILDS_DIR:-/srv/mobile-builds}"

WORKFLOW="eas-build.yml"
APP_NAME="TNotes"
BUNDLE_ID="net.toonion.tnotes"

# Validate
if [[ ! "$PLATFORM" =~ ^(android|ios)$ ]]; then
  echo "❌ Invalid platform: ${PLATFORM}"; exit 1
fi
if [[ ! "$PROFILE" =~ ^(development|preview|production)$ ]]; then
  echo "❌ Invalid profile: ${PROFILE}"; exit 1
fi

# Resolve repo
REPO=$(gh repo view --json nameWithOwner --jq '.nameWithOwner' 2>/dev/null || echo "")
if [[ -z "$REPO" ]]; then
  echo "❌ Could not resolve repo. Run this from the tnotes repo directory."
  exit 1
fi

# Find the latest successful run for this platform+profile
echo "🔍 Finding latest ${PLATFORM} ${PROFILE} build..."
RUN_ID=$(gh run list \
  --repo "${REPO}" \
  --workflow "${WORKFLOW}" \
  --status success \
  --limit 20 \
  --json databaseId,displayTitle \
  --jq "[.[] | select(.displayTitle | test(\"${PLATFORM}\"; \"i\"))] | .[0].databaseId // empty")

if [[ -z "$RUN_ID" ]]; then
  echo "❌ No successful ${PLATFORM} build found. Run a build first:"
  echo "   ./scripts/build-mobile.sh ${PLATFORM} ${PROFILE}"
  exit 1
fi

echo "📋 Found run #${RUN_ID}"

# Download artifact
ARTIFACT_NAME="${PLATFORM}-${PROFILE}-build"
DOWNLOAD_DIR=$(mktemp -d)
echo "📦 Downloading artifact '${ARTIFACT_NAME}'..."
gh run download "${RUN_ID}" \
  --repo "${REPO}" \
  --name "${ARTIFACT_NAME}" \
  --dir "${DOWNLOAD_DIR}" 2>/dev/null || {
    echo "❌ Artifact '${ARTIFACT_NAME}' not found in run #${RUN_ID}."
    echo "   Available artifacts:"
    gh run view "${RUN_ID}" --repo "${REPO}" --json artifacts --jq '.artifacts[].name'
    rm -rf "${DOWNLOAD_DIR}"
    exit 1
  }

# Setup serve directory
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
SERVE_DIR="${BUILDS_DIR}/${PLATFORM}/${PROFILE}"
mkdir -p "${SERVE_DIR}"

BASE_URL="https://${BUILDS_DOMAIN}/${PLATFORM}/${PROFILE}"

if [[ "$PLATFORM" == "ios" ]]; then
  # Move .ipa
  IPA_FILE=$(find "${DOWNLOAD_DIR}" -name "*.ipa" | head -1)
  if [[ -z "$IPA_FILE" ]]; then
    echo "❌ No .ipa found in artifact"; rm -rf "${DOWNLOAD_DIR}"; exit 1
  fi
  cp "${IPA_FILE}" "${SERVE_DIR}/tnotes.ipa"

  # Read version from app.json if available
  APP_VERSION="0.1.0"
  if [[ -f "mobile/app.json" ]]; then
    APP_VERSION=$(grep -o '"version": *"[^"]*"' mobile/app.json | head -1 | cut -d'"' -f4 || echo "0.1.0")
  fi

  # Generate manifest.plist
  cat > "${SERVE_DIR}/manifest.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>items</key>
  <array>
    <dict>
      <key>assets</key>
      <array>
        <dict>
          <key>kind</key>
          <string>software-package</string>
          <key>url</key>
          <string>${BASE_URL}/tnotes.ipa</string>
        </dict>
      </array>
      <key>metadata</key>
      <dict>
        <key>bundle-identifier</key>
        <string>${BUNDLE_ID}</string>
        <key>bundle-version</key>
        <string>${APP_VERSION}</string>
        <key>kind</key>
        <string>software</string>
        <key>title</key>
        <string>${APP_NAME}</string>
      </dict>
    </dict>
  </array>
</dict>
</plist>
PLIST

  INSTALL_URL="itms-services://?action=download-manifest&url=${BASE_URL}/manifest.plist"
  DISPLAY_URL="${BASE_URL}"

elif [[ "$PLATFORM" == "android" ]]; then
  # Move .apk
  APK_FILE=$(find "${DOWNLOAD_DIR}" -name "*.apk" | head -1)
  if [[ -z "$APK_FILE" ]]; then
    echo "❌ No .apk found in artifact"; rm -rf "${DOWNLOAD_DIR}"; exit 1
  fi
  cp "${APK_FILE}" "${SERVE_DIR}/tnotes.apk"

  INSTALL_URL="${BASE_URL}/tnotes.apk"
  DISPLAY_URL="${INSTALL_URL}"
fi

# Generate a simple install page
cat > "${SERVE_DIR}/index.html" <<HTML
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Install ${APP_NAME}</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { font-family: -apple-system, system-ui, sans-serif; background: #141318; color: #fff;
           display: flex; justify-content: center; align-items: center; min-height: 100vh; padding: 2rem; }
    .card { text-align: center; max-width: 400px; }
    h1 { font-size: 1.5rem; margin-bottom: 0.5rem; }
    .meta { color: #888; font-size: 0.875rem; margin-bottom: 2rem; }
    .btn { display: inline-block; padding: 1rem 2rem; background: #6750a4; color: #fff;
           text-decoration: none; border-radius: 12px; font-size: 1.1rem; font-weight: 600; }
    .btn:active { background: #7c68b5; }
  </style>
</head>
<body>
  <div class="card">
    <h1>📱 ${APP_NAME}</h1>
    <p class="meta">${PLATFORM} · ${PROFILE} · $(date '+%b %d, %Y %H:%M')</p>
    <a class="btn" href="${INSTALL_URL}">Install ${APP_NAME}</a>
  </div>
</body>
</html>
HTML

# Cleanup
rm -rf "${DOWNLOAD_DIR}"

echo ""
echo "✅ Deployed ${PLATFORM} ${PROFILE} build!"
echo ""
echo "📂 Files:  ${SERVE_DIR}/"
echo "🌐 Page:   ${DISPLAY_URL}"
echo "📲 Install: ${INSTALL_URL}"
echo ""

# Print QR code if qrencode is available
if command -v qrencode &>/dev/null; then
  echo "📱 Scan to install:"
  echo ""
  qrencode -t ansiutf8 "${DISPLAY_URL}"
else
  echo "💡 Install qrencode for a terminal QR code: sudo apt install qrencode"
fi
