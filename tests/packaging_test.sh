#!/usr/bin/env bash
# Integration test for scripts/package.sh
# Run manually or in CI after a successful build.
# Does NOT run as part of `cargo test`.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

PASS=0
FAIL=0

ok()   { echo "  ✓ $1"; PASS=$((PASS + 1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL + 1)); }

echo "=== Packaging Integration Test ==="
echo ""

echo "Step 1/3: Building plugin..."
bash scripts/build.sh

echo ""
echo "Step 2/3: Running package.sh..."
PLUGIN_VERSION="test-0.0.1" bash scripts/package.sh

echo ""
echo "Step 3/3: Verifying tarball..."

OS="$(uname -s)"
ARCH="$(uname -m)"
case "${OS}" in
  Linux)  PLATFORM_OS="linux";  BINARY="libbmad_plugin.so" ;;
  Darwin) PLATFORM_OS="darwin"; BINARY="libbmad_plugin.dylib" ;;
esac
case "${ARCH}" in
  x86_64)         PLATFORM_ARCH="x86_64" ;;
  aarch64|arm64)  PLATFORM_ARCH="aarch64" ;;
esac
PLATFORM="${PLATFORM_OS}-${PLATFORM_ARCH}"
ARCHIVE_NAME="bmad-method-test-0.0.1-${PLATFORM}"
TARBALL="dist/${ARCHIVE_NAME}.tar.gz"

if [[ ! -f "${TARBALL}" ]]; then
  echo "FATAL: Tarball not found: ${TARBALL}" >&2
  exit 1
fi

CONTENTS="$(tar -tzf "${TARBALL}")"

echo "${CONTENTS}" | grep -qF "${ARCHIVE_NAME}/${BINARY}"    && ok "binary present"    || fail "binary missing"
echo "${CONTENTS}" | grep -qF "${ARCHIVE_NAME}/plugin.toml"  && ok "plugin.toml present" || fail "plugin.toml missing"
echo "${CONTENTS}" | grep -qF "${ARCHIVE_NAME}/README.md"    && ok "README.md present"  || fail "README.md missing"

EXTRACT_DIR="$(mktemp -d)"
tar -xzf "${TARBALL}" -C "${EXTRACT_DIR}"
TOML_FILE="${EXTRACT_DIR}/${ARCHIVE_NAME}/plugin.toml"

grep -qF 'name = "bmad-method"'      "${TOML_FILE}" && ok 'plugin.toml name = "bmad-method"'        || fail 'plugin.toml name is wrong or missing'
grep -qF 'version = "test-0.0.1"'    "${TOML_FILE}" && ok 'plugin.toml version = "test-0.0.1"'       || fail 'plugin.toml version is wrong or missing (AC#4)'
grep -qE 'api_version = [0-9]+'      "${TOML_FILE}" && ok "plugin.toml api_version is integer"       || fail "plugin.toml api_version missing or not an integer (AC#3)"
grep -qE 'api_version = [1-9][0-9]*' "${TOML_FILE}" && ok "plugin.toml api_version is non-zero"      || fail "plugin.toml api_version is zero — PLUGIN_API_VERSION must be >= 1 (AC#3)"
grep -qE 'agent_count = [1-9][0-9]*' "${TOML_FILE}" && ok "plugin.toml agent_count is positive int"  || fail "plugin.toml agent_count is zero or missing"

rm -rf "${EXTRACT_DIR}"

echo ""
echo "Results: ${PASS} passed, ${FAIL} failed"
[[ "${FAIL}" -eq 0 ]] && echo "✓ All checks passed" || { echo "✗ Some checks failed" >&2; exit 1; }
