#!/usr/bin/env bash
# Manual Installation Acceptance Test
#
# Simulates a manual installation of the bmad-method plugin by downloading
# the latest release tarball from GitHub Releases, placing the files in the
# Pulse plugin directory, and verifying that Pulse loads the plugin correctly.
#
# REQUIREMENTS:
#   - Pulse v0.9.0+ installed and on PATH
#   - Internet access (to download the release tarball from GitHub)
#   - curl or wget available
#   - The GITHUB_REPO variable below must point to the correct repository
#
# This test is NOT run in CI — it requires a live Pulse installation and
# network access. Run it manually to validate the manual installation path.
#
# Usage:
#   bash tests/manual_install_test.sh
#   GITHUB_REPO=myorg/bmad-method bash tests/manual_install_test.sh

set -euo pipefail

GITHUB_REPO="${GITHUB_REPO:-<YOUR-ORG>/bmad-method}"
PLUGIN_DIR="${HOME}/.pulse/plugins/bmad-method"

PASS=0
FAIL=0
SKIPPED=0

pass()  { echo "  ✓ $1"; PASS=$((PASS + 1)); }
fail()  { echo "  ✗ $1"; FAIL=$((FAIL + 1)); }
skip()  { echo "  - $1 (skipped: $2)"; SKIPPED=$((SKIPPED + 1)); }

echo "=== BMAD-METHOD Manual Installation Acceptance Test ==="
echo "Repo:        ${GITHUB_REPO}"
echo "Plugin dir:  ${PLUGIN_DIR}"
echo ""

if ! command -v pulse >/dev/null 2>&1; then
  echo "ERROR: 'pulse' command not found."
  echo "       Install Pulse v0.9.0+ before running this test."
  echo "       See: https://pulse.sh/docs/installation"
  exit 1
fi
echo "Pulse:       $(pulse --version 2>/dev/null || echo 'unknown')"
echo ""

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "${WORK_DIR}"' EXIT

echo "--- Step 1: Detect platform ---"
OS="$(uname -s)"
ARCH="$(uname -m)"
case "${OS}" in
  Linux)  PLATFORM_OS="linux";  BINARY="libbmad_plugin.so" ;;
  Darwin) PLATFORM_OS="darwin"; BINARY="libbmad_plugin.dylib" ;;
  *)
    echo "ERROR: Unsupported OS: ${OS}"
    exit 1
    ;;
esac
case "${ARCH}" in
  x86_64)        PLATFORM_ARCH="x86_64" ;;
  aarch64|arm64) PLATFORM_ARCH="aarch64" ;;
  *)
    echo "ERROR: Unsupported architecture: ${ARCH}"
    exit 1
    ;;
esac
PLATFORM="${PLATFORM_OS}-${PLATFORM_ARCH}"
pass "Platform detected: ${PLATFORM}"
echo ""

echo "--- Step 2: Fetch latest release version from GitHub ---"
RELEASES_URL="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
# Preserve any VERSION passed via environment (e.g. VERSION=1.0.0 bash tests/manual_install_test.sh)
VERSION="${VERSION:-}"
if [[ -z "${VERSION}" ]]; then
  if command -v curl >/dev/null 2>&1; then
    VERSION="$(curl -fsSL "${RELEASES_URL}" 2>/dev/null \
      | grep '"tag_name"' \
      | sed 's/.*"tag_name": *"v\{0,1\}\([^"]*\)".*/\1/' \
      | head -1 || true)"
  elif command -v wget >/dev/null 2>&1; then
    VERSION="$(wget -qO- "${RELEASES_URL}" 2>/dev/null \
      | grep '"tag_name"' \
      | sed 's/.*"tag_name": *"v\{0,1\}\([^"]*\)".*/\1/' \
      | head -1 || true)"
  fi
fi

if [[ -z "${VERSION}" ]]; then
  echo "ERROR: Could not determine latest release version from GitHub."
  echo "       Check that GITHUB_REPO=${GITHUB_REPO} is correct and that releases exist."
  echo "       You can set VERSION manually: VERSION=1.0.0 bash tests/manual_install_test.sh"
  exit 1
else
  pass "Latest release version: ${VERSION}"
fi
echo ""

echo "--- Step 3: Download release tarball ---"
ARCHIVE_NAME="bmad-method-${VERSION}-${PLATFORM}"
TARBALL_FILE="${WORK_DIR}/${ARCHIVE_NAME}.tar.gz"
DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${ARCHIVE_NAME}.tar.gz"

echo "  URL: ${DOWNLOAD_URL}"
DOWNLOAD_OK=false
if command -v curl >/dev/null 2>&1; then
  if curl -fsSL -o "${TARBALL_FILE}" "${DOWNLOAD_URL}" 2>/dev/null; then
    DOWNLOAD_OK=true
  fi
elif command -v wget >/dev/null 2>&1; then
  if wget -qO "${TARBALL_FILE}" "${DOWNLOAD_URL}" 2>/dev/null; then
    DOWNLOAD_OK=true
  fi
fi

if [[ "${DOWNLOAD_OK}" == "true" ]] && [[ -f "${TARBALL_FILE}" ]]; then
  pass "Tarball downloaded: ${TARBALL_FILE}"
else
  fail "Failed to download tarball from ${DOWNLOAD_URL}"
  echo ""
  echo "Troubleshooting:"
  echo "  - Check that GITHUB_REPO=${GITHUB_REPO} is correct"
  echo "  - Confirm that release v${VERSION} exists and has ${ARCHIVE_NAME}.tar.gz"
  echo "  - Ensure internet access is available from this machine"
  exit 1
fi
echo ""

echo "--- Step 4: Verify tarball contents ---"
CONTENTS="$(tar -tzf "${TARBALL_FILE}" 2>/dev/null)"
TARBALL_VALID=true
if echo "${CONTENTS}" | grep -qF "${ARCHIVE_NAME}/${BINARY}"; then
  pass "Binary present in tarball: ${BINARY}"
else
  fail "Binary missing from tarball: ${ARCHIVE_NAME}/${BINARY}"
  TARBALL_VALID=false
fi
if echo "${CONTENTS}" | grep -qF "${ARCHIVE_NAME}/plugin.toml"; then
  pass "plugin.toml present in tarball"
else
  fail "plugin.toml missing from tarball"
  TARBALL_VALID=false
fi
if echo "${CONTENTS}" | grep -qF "${ARCHIVE_NAME}/README.md"; then
  pass "README.md present in tarball (optional but expected)"
else
  skip "README.md in tarball" "not required for plugin loading — informational only"
fi
if [[ "${TARBALL_VALID}" == "false" ]]; then
  echo ""
  echo "ERROR: Tarball is missing required files — cannot proceed with installation."
  echo "       Download the correct tarball for platform: ${PLATFORM}"
  exit 1
fi
echo ""

echo "--- Step 5: Extract tarball ---"
tar -xzf "${TARBALL_FILE}" -C "${WORK_DIR}"
if [[ -d "${WORK_DIR}/${ARCHIVE_NAME}" ]]; then
  pass "Tarball extracted to ${WORK_DIR}/${ARCHIVE_NAME}/"
else
  fail "Extraction failed — directory not found: ${WORK_DIR}/${ARCHIVE_NAME}/"
  exit 1
fi
echo ""

echo "--- Step 6: Back up existing plugin installation (if any) ---"
BACKUP_DIR=""
if [[ -d "${PLUGIN_DIR}" ]]; then
  BACKUP_DIR="${WORK_DIR}/bmad-method-backup"
  cp -r "${PLUGIN_DIR}" "${BACKUP_DIR}"
  echo "  Backed up existing plugin to: ${BACKUP_DIR}"
fi
echo ""

echo "--- Step 7: Install plugin files manually ---"
mkdir -p "${PLUGIN_DIR}"
cp "${WORK_DIR}/${ARCHIVE_NAME}/${BINARY}"      "${PLUGIN_DIR}/${BINARY}"
cp "${WORK_DIR}/${ARCHIVE_NAME}/plugin.toml"    "${PLUGIN_DIR}/plugin.toml"

if [[ -f "${PLUGIN_DIR}/${BINARY}" ]]; then
  pass "Binary installed: ${PLUGIN_DIR}/${BINARY}"
else
  fail "Binary not found after copy: ${PLUGIN_DIR}/${BINARY}"
fi
if [[ -f "${PLUGIN_DIR}/plugin.toml" ]]; then
  pass "plugin.toml installed: ${PLUGIN_DIR}/plugin.toml"
else
  fail "plugin.toml not found after copy: ${PLUGIN_DIR}/plugin.toml"
fi
echo ""

echo "--- Step 8: Verify plugin.toml is well-formed ---"
if grep -qF 'name = "bmad-method"' "${PLUGIN_DIR}/plugin.toml"; then
  pass "plugin.toml: name = \"bmad-method\""
else
  fail "plugin.toml: name field missing or wrong"
fi
if grep -qE 'api_version = [0-9]+' "${PLUGIN_DIR}/plugin.toml"; then
  API_VER="$(grep 'api_version' "${PLUGIN_DIR}/plugin.toml" | sed 's/.*= *//')"
  pass "plugin.toml: api_version = ${API_VER} (integer)"
else
  fail "plugin.toml: api_version missing or not an integer"
fi
if grep -qE 'agent_count = [1-9][0-9]*' "${PLUGIN_DIR}/plugin.toml"; then
  AGENT_COUNT="$(grep 'agent_count' "${PLUGIN_DIR}/plugin.toml" | sed 's/.*= *//')"
  pass "plugin.toml: agent_count = ${AGENT_COUNT} (positive)"
else
  fail "plugin.toml: agent_count missing or zero"
fi
echo ""

echo "--- Step 9: Reload Pulse plugins ---"
RELOAD_OK=false
RELOAD_OUTPUT=""
if RELOAD_OUTPUT="$(pulse plugin reload 2>&1)"; then
  RELOAD_OK=true
  pass "pulse plugin reload exited 0"
else
  RELOAD_EXIT=$?
  if echo "${RELOAD_OUTPUT}" | grep -qi "not found\|unknown command\|reload.*unavailable"; then
    skip "pulse plugin reload" "command may not be available in this Pulse version — restart Pulse manually"
  else
    fail "pulse plugin reload failed (exit ${RELOAD_EXIT})"
    echo "      Output: ${RELOAD_OUTPUT}"
  fi
fi
echo ""

echo "--- Step 10: Verify plugin appears in pulse plugin list ---"
LIST_OUTPUT=""
if LIST_OUTPUT="$(pulse plugin list 2>&1)"; then
  if echo "${LIST_OUTPUT}" | grep -q "bmad-method"; then
    pass "bmad-method appears in 'pulse plugin list'"
    if echo "${LIST_OUTPUT}" | grep "bmad-method" | grep -qE "v?[0-9]+\.[0-9]+\.[0-9]+"; then
      pass "version string present in plugin list output"
    else
      skip "version format check" "pulse may format version differently — check manually"
    fi
  else
    fail "bmad-method NOT found in 'pulse plugin list'"
    echo "      Output was:"
    echo "${LIST_OUTPUT}" | sed 's/^/      /'
  fi
else
  fail "pulse plugin list returned non-zero exit code"
  echo "      Output: ${LIST_OUTPUT}"
fi
echo ""

echo "--- Step 11: Verify executor resolves in a minimal workflow ---"
WORKFLOW_FILE="${WORK_DIR}/test-bmad.yaml"
cat > "${WORKFLOW_FILE}" << 'YAML'
workflow:
  name: bmad-manual-install-test
  steps:
    - name: test-architect
      executor: bmad/architect
      input: "Say hello from the Architect agent"
YAML

if WORKFLOW_OUTPUT="$(pulse run "${WORKFLOW_FILE}" 2>&1)"; then
  if echo "${WORKFLOW_OUTPUT}" | grep -qi "not found\|unknown executor\|missing executor\|executor.*unavailable"; then
    fail "Workflow ran but output indicates executor resolution failure"
    echo "      Output: ${WORKFLOW_OUTPUT}"
  else
    pass "bmad/architect executor resolved — workflow ran without error"
  fi
else
  EXIT_CODE=$?
  if echo "${WORKFLOW_OUTPUT}" | grep -qi "not found\|unknown executor\|missing"; then
    fail "bmad/architect executor not found (exit ${EXIT_CODE})"
  else
    fail "Workflow execution failed (exit ${EXIT_CODE})"
  fi
  echo "      Output: ${WORKFLOW_OUTPUT}"
fi
echo ""

echo "--- Step 12: Restore previous plugin installation (if backed up) ---"
if [[ -n "${BACKUP_DIR}" ]] && [[ -d "${BACKUP_DIR}" ]]; then
  RESTORE_DEST="${PLUGIN_DIR}"
  cp -r "${BACKUP_DIR}" "${RESTORE_DEST}.tmp" \
    && rm -rf "${RESTORE_DEST}" \
    && mv "${RESTORE_DEST}.tmp" "${RESTORE_DEST}" \
    && echo "  Restored previous installation from backup." \
    || echo "  WARNING: Restore failed — manually reinstall from ${BACKUP_DIR} before it is cleaned up."
  pulse plugin reload >/dev/null 2>&1 || true
fi
echo ""

echo "==========================================="
echo "Results: ${PASS} passed, ${FAIL} failed, ${SKIPPED} skipped"
echo ""
if [[ ${FAIL} -gt 0 ]]; then
  echo "RESULT: FAILED"
  echo ""
  echo "Troubleshooting:"
  echo "  1. Ensure Pulse v0.9.0+ is installed: pulse --version"
  echo "  2. Ensure GITHUB_REPO=${GITHUB_REPO} is correct and releases exist"
  echo "  3. On macOS, if the plugin fails to load, remove quarantine:"
  echo "     xattr -d com.apple.quarantine ${PLUGIN_DIR}/libbmad_plugin.dylib"
  echo "  4. Check file permissions: ls -la ${PLUGIN_DIR}/"
  exit 1
else
  echo "RESULT: PASSED"
  exit 0
fi
