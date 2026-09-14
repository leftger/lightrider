#!/usr/bin/env bash
set -euo pipefail

# Build script for Windows target using cargo-xwin on Linux.
# Produces a self-contained release zip with lightrider.exe and assets/.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
TARGET="x86_64-pc-windows-msvc"
DIST_DIR="${ROOT_DIR}/dist"
PACKAGE_NAME="lightrider-windows-x86_64"

echo "=== Building Lightrider for Windows ($TARGET) ==="

cd "${ROOT_DIR}"

# 1. Verify Rust target
if ! rustup target list | grep -q "${TARGET} (installed)"; then
    echo ">> Installing Rust target ${TARGET}..."
    rustup target add "${TARGET}"
fi

# 2. Verify cargo-xwin
if ! command -v cargo-xwin >/dev/null 2>&1; then
    echo ">> cargo-xwin not found. Installing cargo-xwin..."
    if command -v cargo-binstall >/dev/null 2>&1; then
        cargo binstall cargo-xwin --no-confirm
    else
        cargo install cargo-xwin --locked
    fi
fi

# 3. Build release binary
echo ">> Compiling release binary with cargo-xwin..."
cargo xwin build --target "${TARGET}" --release

# 4. Package distribution zip
echo ">> Packaging distribution bundle..."
rm -rf "${DIST_DIR}/${PACKAGE_NAME}" "${DIST_DIR}/${PACKAGE_NAME}.zip"
mkdir -p "${DIST_DIR}/${PACKAGE_NAME}"

cp "target/${TARGET}/release/lightrider.exe" "${DIST_DIR}/${PACKAGE_NAME}/"
cp -r assets "${DIST_DIR}/${PACKAGE_NAME}/"
cp README.md LICENSE "${DIST_DIR}/${PACKAGE_NAME}/" 2>/dev/null || true

cd "${DIST_DIR}"
zip -r "${PACKAGE_NAME}.zip" "${PACKAGE_NAME}" >/dev/null

echo "=== Windows build complete! ==="
echo "Binary : ${DIST_DIR}/${PACKAGE_NAME}/lightrider.exe"
echo "Package: ${DIST_DIR}/${PACKAGE_NAME}.zip"
