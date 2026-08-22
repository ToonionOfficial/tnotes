#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

echo "Building TNotes web frontend..."
cd "${REPO_ROOT}/web"
pnpm install --frozen-lockfile
pnpm run build

echo "Compiling release server binary..."
cd "${REPO_ROOT}"
cargo build --release -p tnotes-server

BINARY_PATH="${REPO_ROOT}/target/release/tnotes-server"
BINARY_SIZE="$(ls -lh "${BINARY_PATH}" | awk '{print $5}')"

echo "Build complete: ${BINARY_PATH} (${BINARY_SIZE})"
