#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

echo "Building Notat web frontend..."
cd "${REPO_ROOT}/web"
pnpm install --frozen-lockfile
pnpm run build

echo "Compiling release server binary..."
cd "${REPO_ROOT}"
cargo build --release -p notat-server

BINARY_PATH="${REPO_ROOT}/target/release/notat-server"
BINARY_SIZE="$(ls -lh "${BINARY_PATH}" | awk '{print $5}')"

echo "Build complete: ${BINARY_PATH} (${BINARY_SIZE})"
