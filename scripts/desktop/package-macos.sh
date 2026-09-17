#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

ARCH="${1:-arm64}"
VERSION="${2:-$(grep '^version =' "${REPO_ROOT}/Cargo.toml" | head -n1 | cut -d '"' -f2)}"
if [ -z "${VERSION}" ]; then
    VERSION="$(grep '^version =' "${REPO_ROOT}/apps/desktop/Cargo.toml" | head -n1 | cut -d '"' -f2)"
fi
DIST_DIR="${REPO_ROOT}/dist"
mkdir -p "${DIST_DIR}"

case "${ARCH}" in
    arm64|aarch64|aarch64-apple-darwin)
        TARGET="aarch64-apple-darwin"
        ARCH_NAME="arm64"
        ;;
    x64|x86_64|x86_64-apple-darwin)
        TARGET="x86_64-apple-darwin"
        ARCH_NAME="x64"
        ;;
    universal)
        TARGET="universal"
        ARCH_NAME="universal"
        ;;
    *)
        echo "Unknown architecture: ${ARCH}. Use arm64 or x64."
        exit 1
        ;;
esac

echo "==> Packaging macOS (${ARCH_NAME}) for TNotes v${VERSION}..."

APP_BUNDLE="${DIST_DIR}/TNotes.app"
rm -rf "${APP_BUNDLE}"
mkdir -p "${APP_BUNDLE}/Contents/MacOS"
mkdir -p "${APP_BUNDLE}/Contents/Resources"

# Determine binary location
if [ "${TARGET}" = "universal" ]; then
    ARM_BIN="${REPO_ROOT}/target/aarch64-apple-darwin/release/tnotes-desktop"
    X64_BIN="${REPO_ROOT}/target/x86_64-apple-darwin/release/tnotes-desktop"
    if [ ! -f "${ARM_BIN}" ] || [ ! -f "${X64_BIN}" ]; then
        echo "Both arm64 and x64 release binaries must exist to build a universal bundle."
        exit 1
    fi
    echo "--> Creating universal binary with lipo..."
    lipo -create -output "${APP_BUNDLE}/Contents/MacOS/tnotes-desktop" "${ARM_BIN}" "${X64_BIN}"
else
    SRC_BIN="${REPO_ROOT}/target/${TARGET}/release/tnotes-desktop"
    if [ ! -f "${SRC_BIN}" ]; then
        SRC_BIN="${REPO_ROOT}/target/release/tnotes-desktop"
    fi
    if [ ! -f "${SRC_BIN}" ]; then
        echo "--> Binary not found. Building target ${TARGET}..."
        cargo build --release --target "${TARGET}" -p tnotes-desktop
        SRC_BIN="${REPO_ROOT}/target/${TARGET}/release/tnotes-desktop"
    fi
    cp "${SRC_BIN}" "${APP_BUNDLE}/Contents/MacOS/tnotes-desktop"
fi

chmod +x "${APP_BUNDLE}/Contents/MacOS/tnotes-desktop"
strip "${APP_BUNDLE}/Contents/MacOS/tnotes-desktop" 2>/dev/null || true

# Generate Info.plist
cat > "${APP_BUNDLE}/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>TNotes</string>
    <key>CFBundleExecutable</key>
    <string>tnotes-desktop</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.toonion.tnotes</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>TNotes</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
EOF

echo -n "APPL????" > "${APP_BUNDLE}/Contents/PkgInfo"

# Generate ICNS if on macOS with iconutil and sips
ICON_PNG="${REPO_ROOT}/apps/desktop/assets/app-icon.png"
if command -v iconutil >/dev/null 2>&1 && command -v sips >/dev/null 2>&1 && [ -f "${ICON_PNG}" ]; then
    echo "--> Generating macOS AppIcon.icns..."
    ICONSET_DIR="${DIST_DIR}/AppIcon.iconset"
    rm -rf "${ICONSET_DIR}"
    mkdir -p "${ICONSET_DIR}"

    sips -z 16 16     "${ICON_PNG}" --out "${ICONSET_DIR}/icon_16x16.png" >/dev/null
    sips -z 32 32     "${ICON_PNG}" --out "${ICONSET_DIR}/icon_16x16@2x.png" >/dev/null
    sips -z 32 32     "${ICON_PNG}" --out "${ICONSET_DIR}/icon_32x32.png" >/dev/null
    sips -z 64 64     "${ICON_PNG}" --out "${ICONSET_DIR}/icon_32x32@2x.png" >/dev/null
    sips -z 128 128   "${ICON_PNG}" --out "${ICONSET_DIR}/icon_128x128.png" >/dev/null
    sips -z 256 256   "${ICON_PNG}" --out "${ICONSET_DIR}/icon_128x128@2x.png" >/dev/null
    sips -z 256 256   "${ICON_PNG}" --out "${ICONSET_DIR}/icon_256x256.png" >/dev/null
    sips -z 512 512   "${ICON_PNG}" --out "${ICONSET_DIR}/icon_256x256@2x.png" >/dev/null
    sips -z 512 512   "${ICON_PNG}" --out "${ICONSET_DIR}/icon_512x512.png" >/dev/null
    sips -z 1024 1024 "${ICON_PNG}" --out "${ICONSET_DIR}/icon_512x512@2x.png" >/dev/null

    iconutil -c icns "${ICONSET_DIR}" -o "${APP_BUNDLE}/Contents/Resources/AppIcon.icns"
    rm -rf "${ICONSET_DIR}"
fi

# Code Signing
if [ -n "${APPLE_SIGNING_IDENTITY:-}" ]; then
    echo "--> Signing bundle with Developer ID: ${APPLE_SIGNING_IDENTITY}..."
    codesign --deep --force --options runtime --sign "${APPLE_SIGNING_IDENTITY}" "${APP_BUNDLE}"
else
    echo "--> Applying ad-hoc code signature..."
    codesign --deep --force -s - "${APP_BUNDLE}" 2>/dev/null || true
fi

# 1. Package ZIP (for in-app updater)
echo "--> Creating update ZIP archive..."
ZIP_OUTPUT="${DIST_DIR}/TNotes-macOS-${ARCH_NAME}.zip"
rm -f "${ZIP_OUTPUT}"
if command -v ditto >/dev/null 2>&1; then
    ditto -c -k --keepParent "${APP_BUNDLE}" "${ZIP_OUTPUT}"
else
    (cd "${DIST_DIR}" && zip -q -r -y "TNotes-macOS-${ARCH_NAME}.zip" "TNotes.app")
fi

# 2. Package DMG (for user installation)
echo "--> Creating DMG disk image..."
DMG_OUTPUT="${DIST_DIR}/TNotes-macOS-${ARCH_NAME}.dmg"
rm -f "${DMG_OUTPUT}"

if command -v create-dmg >/dev/null 2>&1; then
    create-dmg \
        --volname "TNotes Installer" \
        --window-pos 200 120 \
        --window-size 600 400 \
        --icon-size 100 \
        --icon "TNotes.app" 175 120 \
        --hide-extension "TNotes.app" \
        --app-drop-link 425 120 \
        "${DMG_OUTPUT}" \
        "${APP_BUNDLE}" || true
fi

if [ ! -f "${DMG_OUTPUT}" ] && command -v hdiutil >/dev/null 2>&1; then
    DMG_TEMP_DIR="${DIST_DIR}/dmg_staging"
    rm -rf "${DMG_TEMP_DIR}"
    mkdir -p "${DMG_TEMP_DIR}"
    cp -R "${APP_BUNDLE}" "${DMG_TEMP_DIR}/"
    ln -s /Applications "${DMG_TEMP_DIR}/Applications"
    hdiutil create -volname "TNotes" -srcfolder "${DMG_TEMP_DIR}" -ov -format UDZO "${DMG_OUTPUT}"
    rm -rf "${DMG_TEMP_DIR}"
fi

# Notarization (if Apple credentials provided in CI)
if [ -n "${APPLE_NOTARY_KEY:-}" ] && [ -n "${APPLE_NOTARY_KEY_ID:-}" ] && [ -n "${APPLE_NOTARY_ISSUER:-}" ] && [ -f "${DMG_OUTPUT}" ]; then
    echo "--> Submitting DMG for Apple notarization..."
    echo "${APPLE_NOTARY_KEY}" > /tmp/notary_key.p8
    xcrun notarytool submit "${DMG_OUTPUT}" \
        --key /tmp/notary_key.p8 \
        --key-id "${APPLE_NOTARY_KEY_ID}" \
        --issuer "${APPLE_NOTARY_ISSUER}" \
        --wait
    rm -f /tmp/notary_key.p8
    echo "--> Stapling ticket..."
    xcrun stapler staple "${DMG_OUTPUT}"
fi

echo "==> macOS build complete! Generated artifacts:"
ls -lh "${DIST_DIR}"/TNotes-macOS-* 2>/dev/null || true
