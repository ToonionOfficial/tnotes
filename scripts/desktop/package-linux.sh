#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

VERSION="${1:-$(grep '^version =' "${REPO_ROOT}/apps/desktop/Cargo.toml" | head -n1 | cut -d '"' -f2)}"
DIST_DIR="${REPO_ROOT}/dist"
mkdir -p "${DIST_DIR}"

BINARY="${REPO_ROOT}/target/release/tnotes-desktop"
if [ ! -f "${BINARY}" ]; then
    echo "==> Building tnotes-desktop release binary..."
    cargo build --release -p tnotes-desktop
fi

echo "==> Packaging Linux distribution for TNotes v${VERSION}..."

# 1. Build portable tarball
echo "--> Creating tarball release..."
TAR_STAGING="${DIST_DIR}/tnotes-${VERSION}-linux-x86_64"
rm -rf "${TAR_STAGING}"
mkdir -p "${TAR_STAGING}/bin" "${TAR_STAGING}/share/applications" "${TAR_STAGING}/share/icons/hicolor/512x512/apps"

cp "${BINARY}" "${TAR_STAGING}/bin/tnotes-desktop"
chmod +x "${TAR_STAGING}/bin/tnotes-desktop"
cp "${REPO_ROOT}/apps/desktop/assets/tnotes.desktop" "${TAR_STAGING}/share/applications/"
cp "${REPO_ROOT}/apps/desktop/assets/app-icon.png" "${TAR_STAGING}/share/icons/hicolor/512x512/apps/tnotes.png"

# Include install/uninstall helper scripts
cat > "${TAR_STAGING}/install.sh" <<'EOF'
#!/usr/bin/env bash
set -e
PREFIX="${PREFIX:-/usr/local}"
install -d "${PREFIX}/bin" "${PREFIX}/share/applications" "${PREFIX}/share/icons/hicolor/512x512/apps"
install -m 755 bin/tnotes-desktop "${PREFIX}/bin/tnotes-desktop"
install -m 644 share/applications/tnotes.desktop "${PREFIX}/share/applications/tnotes.desktop"
install -m 644 share/icons/hicolor/512x512/apps/tnotes.png "${PREFIX}/share/icons/hicolor/512x512/apps/tnotes.png"
echo "TNotes successfully installed to ${PREFIX}!"
EOF
chmod +x "${TAR_STAGING}/install.sh"

tar -czf "${DIST_DIR}/TNotes-Linux-x86_64.tar.gz" -C "${DIST_DIR}" "tnotes-${VERSION}-linux-x86_64"
rm -rf "${TAR_STAGING}"

# 2. Build .deb package (if dpkg-deb is available)
if command -v dpkg-deb >/dev/null 2>&1; then
    echo "--> Creating .deb package..."
    DEB_DIR="${DIST_DIR}/deb-staging"
    rm -rf "${DEB_DIR}"
    mkdir -p "${DEB_DIR}/DEBIAN"
    mkdir -p "${DEB_DIR}/usr/bin"
    mkdir -p "${DEB_DIR}/usr/share/applications"
    mkdir -p "${DEB_DIR}/usr/share/icons/hicolor/512x512/apps"

    cp "${BINARY}" "${DEB_DIR}/usr/bin/tnotes-desktop"
    chmod 755 "${DEB_DIR}/usr/bin/tnotes-desktop"
    cp "${REPO_ROOT}/apps/desktop/assets/tnotes.desktop" "${DEB_DIR}/usr/share/applications/"
    cp "${REPO_ROOT}/apps/desktop/assets/app-icon.png" "${DEB_DIR}/usr/share/icons/hicolor/512x512/apps/tnotes.png"

    cat > "${DEB_DIR}/DEBIAN/control" <<EOF
Package: tnotes
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Toonion <contact@toonion.net>
Description: Local-first GPU-accelerated markdown notebook engineered for speed and offline reliability.
EOF

    dpkg-deb --build --root-owner-group "${DEB_DIR}" "${DIST_DIR}/tnotes_${VERSION}_amd64.deb"
    rm -rf "${DEB_DIR}"
fi

# 3. Build AppImage
echo "--> Creating AppImage..."
APP_DIR="${DIST_DIR}/AppDir"
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/usr/bin"
mkdir -p "${APP_DIR}/usr/share/applications"
mkdir -p "${APP_DIR}/usr/share/icons/hicolor/512x512/apps"

cp "${BINARY}" "${APP_DIR}/usr/bin/tnotes-desktop"
chmod +x "${APP_DIR}/usr/bin/tnotes-desktop"
cp "${REPO_ROOT}/apps/desktop/assets/tnotes.desktop" "${APP_DIR}/usr/share/applications/tnotes.desktop"
cp "${REPO_ROOT}/apps/desktop/assets/tnotes.desktop" "${APP_DIR}/tnotes.desktop"
cp "${REPO_ROOT}/apps/desktop/assets/app-icon.png" "${APP_DIR}/usr/share/icons/hicolor/512x512/apps/tnotes.png"
cp "${REPO_ROOT}/apps/desktop/assets/app-icon.png" "${APP_DIR}/tnotes.png"
cp "${REPO_ROOT}/apps/desktop/assets/app-icon.png" "${APP_DIR}/.DirIcon"

cat > "${APP_DIR}/AppRun" <<'EOF'
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "${HERE}/usr/bin/tnotes-desktop" "$@"
EOF
chmod +x "${APP_DIR}/AppRun"

# Download or run appimagetool
APPIMAGETOOL="${DIST_DIR}/appimagetool"
if ! command -v appimagetool >/dev/null 2>&1; then
    if [ ! -f "${APPIMAGETOOL}" ]; then
        echo "--> Downloading appimagetool..."
        curl -fsSL -o "${APPIMAGETOOL}" "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
        chmod +x "${APPIMAGETOOL}"
    fi
    APPIMAGETOOL_CMD="${APPIMAGETOOL}"
else
    APPIMAGETOOL_CMD="appimagetool"
fi

# When running inside Docker or non-FUSE environments in CI, use --appimage-extract-and-run
export ARCH=x86_64
APPIMAGE_OUTPUT="${DIST_DIR}/TNotes-Linux-x86_64.AppImage"

if [ -n "${APPIMAGE_EXTRACT_AND_RUN:-}" ] || [ -n "${CI:-}" ]; then
    "${APPIMAGETOOL_CMD}" --appimage-extract-and-run "${APP_DIR}" "${APPIMAGE_OUTPUT}" || \
        ARCH=x86_64 "${APPIMAGETOOL_CMD}" --appimage-extract-and-run "${APP_DIR}" "${APPIMAGE_OUTPUT}"
else
    "${APPIMAGETOOL_CMD}" "${APP_DIR}" "${APPIMAGE_OUTPUT}" || \
        ARCH=x86_64 "${APPIMAGETOOL_CMD}" --appimage-extract-and-run "${APP_DIR}" "${APPIMAGE_OUTPUT}"
fi

rm -rf "${APP_DIR}"

echo "==> Linux build complete! Generated artifacts:"
ls -lh "${DIST_DIR}"/TNotes-Linux-* "${DIST_DIR}"/*.deb 2>/dev/null || true
