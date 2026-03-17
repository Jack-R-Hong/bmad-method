#!/usr/bin/env bash
# Usage: ./scripts/package.sh
# Env vars:
#   PLUGIN_VERSION=1.0.0   (optional, defaults to Cargo.toml version)
#   BINARY_PATH=target/x86_64-unknown-linux-gnu/release/libbmad_plugin.so
#                          (optional, overrides auto-detected binary path for cross-compilation)
#   PLATFORM=linux-x86_64  (optional, overrides uname-based platform detection)
# Output: dist/bmad-method-{version}-{platform}.tar.gz

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

STAGING_DIR=""
_CARGO_STDERR="$(mktemp)"
trap 'rm -f "${_CARGO_STDERR}"; [[ -n "${STAGING_DIR}" && -d "${STAGING_DIR}" ]] && rm -rf "${STAGING_DIR}"' EXIT

# --- Platform detection ---
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
  Linux)  PLATFORM_OS="linux" ;;
  Darwin) PLATFORM_OS="darwin" ;;
  *)
    echo "ERROR: Unsupported OS: ${OS}" >&2
    exit 1
    ;;
esac

case "${ARCH}" in
  x86_64)         PLATFORM_ARCH="x86_64" ;;
  aarch64|arm64)  PLATFORM_ARCH="aarch64" ;;
  *)
    echo "ERROR: Unsupported architecture: ${ARCH}" >&2
    exit 1
    ;;
esac

PLATFORM="${PLATFORM:-${PLATFORM_OS}-${PLATFORM_ARCH}}"

# --- Version detection ---
PLUGIN_VERSION="${PLUGIN_VERSION:-$(cargo metadata --no-deps --format-version 1 \
  | python3 -c "import sys,json; pkgs=json.load(sys.stdin)['packages']; \
    print(next(p['version'] for p in pkgs if p['name']=='bmad-plugin'))")}"

if [[ -z "${PLUGIN_VERSION}" ]]; then
  echo "ERROR: Could not determine PLUGIN_VERSION" >&2
  exit 1
fi

# --- Binary name ---
case "${OS}" in
  Linux)  BINARY_NAME="libbmad_plugin.so" ;;
  Darwin) BINARY_NAME="libbmad_plugin.dylib" ;;
  *)
    echo "ERROR: Unsupported OS for binary name: ${OS}" >&2
    exit 1
    ;;
esac

BINARY_PATH="${BINARY_PATH:-${REPO_ROOT}/target/release/${BINARY_NAME}}"
if [[ "${BINARY_PATH}" != /* ]]; then
  BINARY_PATH="${REPO_ROOT}/${BINARY_PATH}"
fi
BINARY_NAME="$(basename "${BINARY_PATH}")"

# --- Pre-flight checks ---
if [[ ! -f "${BINARY_PATH}" ]]; then
  echo "ERROR: Plugin binary not found: ${BINARY_PATH}" >&2
  echo "       Run './scripts/build.sh' first to compile the plugin." >&2
  exit 1
fi

echo "Reading API version from compiled binary..."
API_VERSION="$(cargo run -p bmad-plugin --bin print_api_version --quiet 2>"${_CARGO_STDERR}" || true)"
if [[ -z "${API_VERSION}" ]]; then
  echo "ERROR: Could not read PLUGIN_API_VERSION from print_api_version binary" >&2
  cat "${_CARGO_STDERR}" >&2
  exit 1
fi
if [[ ! "${API_VERSION}" =~ ^[0-9]+$ ]]; then
  echo "ERROR: PLUGIN_API_VERSION is not a non-negative integer: '${API_VERSION}'" >&2
  exit 1
fi

echo "Reading agent count from compiled binary..."
AGENT_COUNT="$(cargo run -p bmad-plugin --bin print_agent_count --quiet 2>"${_CARGO_STDERR}" || true)"
if [[ -z "${AGENT_COUNT}" ]]; then
  echo "ERROR: Could not read agent count from print_agent_count binary" >&2
  cat "${_CARGO_STDERR}" >&2
  exit 1
fi
if [[ ! "${AGENT_COUNT}" =~ ^[0-9]+$ ]]; then
  echo "ERROR: agent_count is not a non-negative integer: '${AGENT_COUNT}'" >&2
  exit 1
fi

ARCHIVE_NAME="bmad-method-${PLUGIN_VERSION}-${PLATFORM}"
STAGING_DIR="${REPO_ROOT}/dist/staging/${ARCHIVE_NAME}"
TARBALL="${REPO_ROOT}/dist/${ARCHIVE_NAME}.tar.gz"

echo "=== BMAD-METHOD Packaging ==="
echo "Version:     ${PLUGIN_VERSION}"
echo "Platform:    ${PLATFORM}"
echo "API version: ${API_VERSION}"
echo "Agents:      ${AGENT_COUNT}"
echo ""

# --- Create staging directory ---
rm -rf "${STAGING_DIR}"
mkdir -p "${STAGING_DIR}"

# --- Generate plugin.toml ---
cat > "${STAGING_DIR}/plugin.toml" << TOML
[plugin]
name = "bmad-method"
version = "${PLUGIN_VERSION}"
api_version = ${API_VERSION}
agent_count = ${AGENT_COUNT}
TOML

# --- Copy files into staging ---
cp "${BINARY_PATH}" "${STAGING_DIR}/${BINARY_NAME}"
cp "${REPO_ROOT}/README.md" "${STAGING_DIR}/README.md"

# --- Create tarball ---
tar -czf "${TARBALL}" -C "${REPO_ROOT}/dist/staging" "${ARCHIVE_NAME}"

# --- Verify tarball contents ---
echo "Verifying tarball contents..."
TARBALL_FILES="$(tar -tzf "${TARBALL}")"

REQUIRED_FILES=(
  "${ARCHIVE_NAME}/${BINARY_NAME}"
  "${ARCHIVE_NAME}/plugin.toml"
  "${ARCHIVE_NAME}/README.md"
)

for REQUIRED in "${REQUIRED_FILES[@]}"; do
  if ! echo "${TARBALL_FILES}" | grep -qF "${REQUIRED}"; then
    echo "ERROR: Required file missing from tarball: ${REQUIRED}" >&2
    rm -rf "${STAGING_DIR}"
    exit 1
  fi
done

# --- Cleanup staging ---
rm -rf "${REPO_ROOT}/dist/staging"

echo "✓ Packaged: dist/${ARCHIVE_NAME}.tar.gz"
