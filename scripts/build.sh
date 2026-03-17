#!/usr/bin/env bash
# BMAD-METHOD Pulse Plugin Build Script
# Usage: ./scripts/build.sh
# Runs the converter then builds the plugin binary.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
AGENTS_DIR="${REPO_ROOT}/agents"
GENERATED_DIR="${REPO_ROOT}/crates/bmad-plugin/src/generated"

# Always run cargo commands from the workspace root so that Cargo can find
# Cargo.toml regardless of where the caller invoked this script.
cd "${REPO_ROOT}"

echo "=== BMAD-METHOD Build Pipeline ==="
echo ""

# Step 1: Run the converter
echo "Step 1/2: Running bmad-converter..."
cargo run -p bmad-converter -- \
    --input "${AGENTS_DIR}" \
    --output "${GENERATED_DIR}"
echo ""

# Step 2: Build the plugin
echo "Step 2/2: Building bmad-plugin..."
cargo build -p bmad-plugin --release
echo ""

# Report output
if [[ "$(uname)" == "Darwin" ]]; then
    PLUGIN_FILE="${REPO_ROOT}/target/release/libbmad_plugin.dylib"
else
    PLUGIN_FILE="${REPO_ROOT}/target/release/libbmad_plugin.so"
fi

if [[ -f "${PLUGIN_FILE}" ]]; then
    SIZE=$(du -sh "${PLUGIN_FILE}" | cut -f1)
    echo "=== Build complete ==="
    echo "Plugin: ${PLUGIN_FILE}"
    echo "Size:   ${SIZE}"
else
    echo "ERROR: Plugin binary not found at expected path: ${PLUGIN_FILE}" >&2
    exit 1
fi
