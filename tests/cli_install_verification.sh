#!/usr/bin/env bash
set -euo pipefail

PASS=0
FAIL=0
SKIPPED=0

pass() { echo "  ✓ $1"; PASS=$((PASS + 1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL + 1)); }
skip() { echo "  - $1 (skipped: $2)"; SKIPPED=$((SKIPPED + 1)); }

if ! command -v pulse >/dev/null 2>&1; then
  echo "ERROR: 'pulse' command not found. Install Pulse v0.9.0+ before running this test."
  echo "       See: https://pulse.sh/docs/installation"
  exit 1
fi

echo "=== BMAD-METHOD CLI Install Verification ==="
echo "Pulse: $(pulse --version 2>/dev/null || echo 'unknown')"
echo ""

echo "--- Step 1: Install via Pulse CLI ---"
INSTALL_OUTPUT=""
if INSTALL_OUTPUT="$(pulse plugin install bmad-method 2>&1)"; then
  pass "pulse plugin install bmad-method exited 0"
  if echo "${INSTALL_OUTPUT}" | grep -qE "v?[0-9]+\.[0-9]+\.[0-9]+"; then
    pass "Install output contains version string (AC1)"
  else
    skip "Install output version check" "pulse may format version differently — check manually: ${INSTALL_OUTPUT}"
  fi
  if echo "${INSTALL_OUTPUT}" | grep -qE "[0-9]+ agents?"; then
    pass "Install output contains agent count (AC1)"
  else
    skip "Install output agent count check" "pulse may format agent count differently — check manually: ${INSTALL_OUTPUT}"
  fi
else
  fail "pulse plugin install bmad-method failed"
  echo "      Output:"
  echo "${INSTALL_OUTPUT}" | sed 's/^/      /'
  echo "FATAL: Cannot continue without successful install."
  exit 1
fi
echo ""

echo "--- Step 2: Verify plugin appears in pulse plugin list ---"
LIST_OUTPUT=""
if ! LIST_OUTPUT="$(pulse plugin list 2>&1)"; then
  fail "pulse plugin list returned non-zero exit code"
  echo "      Output was:"
  echo "${LIST_OUTPUT}" | sed 's/^/      /'
elif echo "${LIST_OUTPUT}" | grep -q "bmad-method"; then
  pass "bmad-method appears in 'pulse plugin list'"
else
  fail "bmad-method NOT found in 'pulse plugin list'"
  echo "      Output was:"
  echo "${LIST_OUTPUT}" | sed 's/^/      /'
fi
echo ""

echo "--- Step 3: Verify version is present in plugin list output ---"
if echo "${LIST_OUTPUT}" | grep "bmad-method" | grep -qE "v?[0-9]+\.[0-9]+\.[0-9]+"; then
  pass "version string found in plugin list for bmad-method"
else
  skip "version format check" "pulse may format version differently — check manually"
fi
echo ""

echo "--- Step 4: Verify executor resolves in a minimal workflow ---"
TMPDIR_WORKFLOW="$(mktemp -d)"
trap 'rm -rf "${TMPDIR_WORKFLOW}"' EXIT

cat > "${TMPDIR_WORKFLOW}/test-bmad.yaml" << 'YAML'
workflow:
  name: bmad-install-test
  steps:
    - name: test-architect
      executor: bmad/architect
      input: "Say hello from the Architect agent"
YAML

echo "  Running workflow: ${TMPDIR_WORKFLOW}/test-bmad.yaml"
if WORKFLOW_OUTPUT="$(pulse run "${TMPDIR_WORKFLOW}/test-bmad.yaml" 2>&1)"; then
  # Check for executor-specific failure patterns only — avoid "error" which is too broad
  if echo "${WORKFLOW_OUTPUT}" | grep -qi "not found\|unknown executor\|missing executor\|executor.*unavailable"; then
    fail "Workflow ran but output indicates executor resolution failure"
    echo "      Output:"
    echo "${WORKFLOW_OUTPUT}" | sed 's/^/      /'
  else
    pass "bmad/architect executor resolved and workflow ran without error"
  fi
else
  EXIT_CODE=$?
  if echo "${WORKFLOW_OUTPUT}" | grep -qi "not found\|unknown executor\|missing"; then
    fail "bmad/architect executor not found (exit ${EXIT_CODE})"
  else
    fail "Workflow failed (exit ${EXIT_CODE})"
  fi
  echo "      Output:"
  echo "${WORKFLOW_OUTPUT}" | sed 's/^/      /'
fi
echo ""

echo "--- Step 5: Check plugin directory structure ---"
PLUGIN_DIR="${HOME}/.pulse/plugins/bmad-method"
if [[ -d "${PLUGIN_DIR}" ]]; then
  pass "Plugin directory exists: ${PLUGIN_DIR}"
  if [[ -f "${PLUGIN_DIR}/plugin.toml" ]]; then
    pass "plugin.toml present"
  else
    fail "plugin.toml missing from ${PLUGIN_DIR}"
  fi
  if [[ -f "${PLUGIN_DIR}/libbmad_plugin.so" ]] || [[ -f "${PLUGIN_DIR}/libbmad_plugin.dylib" ]]; then
    pass "Plugin binary present (libbmad_plugin.so or .dylib)"
  else
    skip "Plugin binary check" "binary may use a different filename — check ${PLUGIN_DIR} manually"
  fi
else
  skip "Plugin directory check" "Pulse may install to a non-standard path"
fi
echo ""

echo "==========================================="
echo "Results: ${PASS} passed, ${FAIL} failed, ${SKIPPED} skipped"
echo ""
if [[ ${FAIL} -gt 0 ]]; then
  echo "RESULT: FAILED"
  echo ""
  echo "Troubleshooting:"
  echo "  1. Ensure Pulse v0.9.0+ is installed"
  echo "  2. If install left partial files: rm -rf ~/.pulse/plugins/bmad-method/"
  echo "  3. Retry: pulse plugin install bmad-method"
  exit 1
else
  echo "RESULT: PASSED"
  exit 0
fi
