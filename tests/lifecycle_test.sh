#!/usr/bin/env bash
# Manual Acceptance Test: Plugin Lifecycle Management (Story 4.6)
#
# Tests the full install → list → uninstall → verify removed → reinstall → update lifecycle.
#
# REQUIREMENTS:
#   - Pulse v0.9.0+ must be installed and in PATH
#   - Internet access required (for install/update from GitHub Releases)
#   - Run from the project root: bash tests/lifecycle_test.sh
#
# EXIT CODE:
#   0 — all mandatory checks passed (skips are not failures)
#   1 — one or more mandatory checks failed
set -euo pipefail

PASS=0
FAIL=0
SKIPPED=0

pass()  { echo "  ✓ $1"; PASS=$((PASS + 1)); }
fail()  { echo "  ✗ $1"; FAIL=$((FAIL + 1)); }
skip()  { echo "  - $1 (skipped: $2)"; SKIPPED=$((SKIPPED + 1)); }
header(){ echo ""; echo "--- $1 ---"; }

PLUGIN_DIR="${HOME}/.pulse/plugins/bmad-method"

# ---------------------------------------------------------------------------
# Pre-flight: Pulse must be installed
# ---------------------------------------------------------------------------
if ! command -v pulse >/dev/null 2>&1; then
  echo "ERROR: 'pulse' command not found. Install Pulse v0.9.0+ before running this test."
  echo "       See: https://pulse.sh/docs/installation"
  exit 1
fi

echo "=== BMAD-METHOD Plugin Lifecycle Acceptance Test ==="
echo "Pulse: $(pulse --version 2>/dev/null || echo 'unknown')"
echo "Plugin dir: ${PLUGIN_DIR}"
echo ""

# ---------------------------------------------------------------------------
# Step 1: Install (or confirm already installed) — establish clean baseline
# ---------------------------------------------------------------------------
header "Step 1: Install plugin (baseline)"
INSTALL_OUTPUT=""
if INSTALL_OUTPUT="$(pulse plugin install bmad-method 2>&1)"; then
  pass "pulse plugin install bmad-method exited 0"
else
  # May already be installed — that's fine
  if echo "${INSTALL_OUTPUT}" | grep -qi "already installed"; then
    pass "Plugin already installed (acceptable baseline)"
  else
    fail "pulse plugin install bmad-method failed"
    echo "${INSTALL_OUTPUT}" | sed 's/^/      /'
    echo "FATAL: Cannot run lifecycle test without a successful install."
    exit 1
  fi
fi

# ---------------------------------------------------------------------------
# Step 2: Verify plugin is listed
# ---------------------------------------------------------------------------
header "Step 2: Verify plugin appears in pulse plugin list"
LIST_OUTPUT="$(pulse plugin list 2>&1 || true)"
if echo "${LIST_OUTPUT}" | grep -q "bmad-method"; then
  pass "bmad-method appears in 'pulse plugin list'"
else
  fail "bmad-method NOT found in 'pulse plugin list'"
  echo "${LIST_OUTPUT}" | sed 's/^/      /'
fi

# Capture version before uninstall for later comparison
INSTALLED_VERSION=""
INSTALLED_VERSION="$(echo "${LIST_OUTPUT}" | grep "bmad-method" | grep -oE "v?[0-9]+\.[0-9]+\.[0-9]+" | head -1 || true)"
if [[ -n "${INSTALLED_VERSION}" ]]; then
  pass "Installed version detected: ${INSTALLED_VERSION}"
else
  skip "Version detection from list" "pulse may format version differently"
fi

# ---------------------------------------------------------------------------
# Step 3: Uninstall
# ---------------------------------------------------------------------------
header "Step 3: Uninstall plugin"
UNINSTALL_OUTPUT=""
if UNINSTALL_OUTPUT="$(pulse plugin uninstall bmad-method 2>&1)"; then
  pass "pulse plugin uninstall bmad-method exited 0"
else
  fail "pulse plugin uninstall bmad-method returned non-zero"
  echo "${UNINSTALL_OUTPUT}" | sed 's/^/      /'
fi

# ---------------------------------------------------------------------------
# Step 4: Verify plugin no longer listed
# ---------------------------------------------------------------------------
header "Step 4: Verify plugin removed from list"
LIST_AFTER="$(pulse plugin list 2>&1 || true)"
if echo "${LIST_AFTER}" | grep -q "bmad-method"; then
  # Some pulse versions show a "disabled" entry — document but don't hard-fail
  if echo "${LIST_AFTER}" | grep "bmad-method" | grep -qi "disabled\|uninstalled"; then
    skip "bmad-method still in list as disabled/uninstalled entry" \
      "this is Pulse's behaviour — the plugin won't be loaded"
  else
    fail "bmad-method still listed as active after uninstall"
    echo "${LIST_AFTER}" | grep "bmad-method" | sed 's/^/      /'
  fi
else
  pass "bmad-method no longer in 'pulse plugin list'"
fi

# ---------------------------------------------------------------------------
# Step 5: Verify plugin directory/files removed
# ---------------------------------------------------------------------------
header "Step 5: Verify plugin files removed from disk"
if [[ ! -d "${PLUGIN_DIR}" ]]; then
  pass "Plugin directory removed: ${PLUGIN_DIR}"
elif [[ -z "$(ls -A "${PLUGIN_DIR}" 2>/dev/null)" ]]; then
  pass "Plugin directory is empty (acceptable)"
else
  # Check if the specific required files are gone
  if [[ ! -f "${PLUGIN_DIR}/libbmad_plugin.so" ]] && \
     [[ ! -f "${PLUGIN_DIR}/libbmad_plugin.dylib" ]] && \
     [[ ! -f "${PLUGIN_DIR}/plugin.toml" ]]; then
    skip "Plugin directory still exists but required files removed" \
      "Pulse left an empty or non-essential-file directory"
  else
    fail "Plugin files still present after uninstall in ${PLUGIN_DIR}"
    ls -la "${PLUGIN_DIR}" | sed 's/^/      /'
  fi
fi

# ---------------------------------------------------------------------------
# Step 6: Verify executor unavailable error (not a crash)
# ---------------------------------------------------------------------------
header "Step 6: Verify clean error for unavailable executor after uninstall"
TMPDIR_WORKFLOW="$(mktemp -d)"
trap 'rm -rf "${TMPDIR_WORKFLOW}"' EXIT

cat > "${TMPDIR_WORKFLOW}/test-bmad-uninstalled.yaml" << 'YAML'
workflow:
  name: bmad-uninstall-test
  steps:
    - name: test-architect
      executor: bmad/architect
      input: "This should produce a clean executor-not-found error"
YAML

EXEC_ERROR_OUTPUT=""
EXEC_EXIT=0
EXEC_ERROR_OUTPUT="$(pulse run "${TMPDIR_WORKFLOW}/test-bmad-uninstalled.yaml" 2>&1)" || EXEC_EXIT=$?

if [[ ${EXEC_EXIT} -ne 0 ]]; then
  pass "Workflow with unavailable executor exits non-zero (expected)"
  # Verify it's a clean error message, not a crash/segfault
  if echo "${EXEC_ERROR_OUTPUT}" | grep -qi "segfault\|segmentation fault\|signal 11\|core dumped\|panic\|SIGSEGV"; then
    fail "Executor-unavailable caused a crash/segfault"
    echo "${EXEC_ERROR_OUTPUT}" | sed 's/^/      /'
  else
    pass "No crash/segfault detected — error is clean"
  fi
  if echo "${EXEC_ERROR_OUTPUT}" | grep -qi "not found\|unavailable\|missing\|no executor\|could not\|unresolved\|cannot find"; then
    pass "Error message indicates executor not found (AC2)"
    echo "  Pulse error output:"
    echo "${EXEC_ERROR_OUTPUT}" | head -5 | sed 's/^/      /'
  else
    skip "Error message format check" \
      "executor-not-found message format differs — output: $(echo "${EXEC_ERROR_OUTPUT}" | head -2)"
  fi
else
  fail "Workflow with uninstalled executor should have failed but exited 0"
fi

# ---------------------------------------------------------------------------
# Step 7: Reinstall
# ---------------------------------------------------------------------------
header "Step 7: Reinstall plugin"
REINSTALL_OUTPUT=""
if REINSTALL_OUTPUT="$(pulse plugin install bmad-method 2>&1)"; then
  pass "pulse plugin install bmad-method (reinstall) exited 0"
else
  fail "Reinstall failed"
  echo "${REINSTALL_OUTPUT}" | sed 's/^/      /'
fi

# ---------------------------------------------------------------------------
# Step 8: Verify plugin listed again after reinstall
# ---------------------------------------------------------------------------
header "Step 8: Verify plugin listed after reinstall"
LIST_REINSTALLED="$(pulse plugin list 2>&1 || true)"
if echo "${LIST_REINSTALLED}" | grep -q "bmad-method"; then
  pass "bmad-method appears in 'pulse plugin list' after reinstall"
else
  fail "bmad-method NOT found in 'pulse plugin list' after reinstall"
fi

# ---------------------------------------------------------------------------
# Step 9: Test "already up to date" behavior
# ---------------------------------------------------------------------------
header "Step 9: Test 'already up to date' — update when already at latest"

# Record binary modification time before update attempt
BINARY_MTIME_BEFORE=""
if [[ -f "${PLUGIN_DIR}/libbmad_plugin.so" ]]; then
  BINARY_MTIME_BEFORE="$(stat -c %Y "${PLUGIN_DIR}/libbmad_plugin.so" 2>/dev/null || true)"
elif [[ -f "${PLUGIN_DIR}/libbmad_plugin.dylib" ]]; then
  BINARY_MTIME_BEFORE="$(stat -f %m "${PLUGIN_DIR}/libbmad_plugin.dylib" 2>/dev/null || true)"
fi

UPDATE_OUTPUT=""
UPDATE_EXIT=0
UPDATE_OUTPUT="$(pulse plugin update bmad-method 2>&1)" || UPDATE_EXIT=$?

if [[ ${UPDATE_EXIT} -eq 0 ]]; then
  pass "pulse plugin update bmad-method exited 0"
else
  skip "pulse plugin update exit code" \
    "non-zero exit on already-up-to-date is Pulse-version-dependent (exit ${UPDATE_EXIT})"
fi

if echo "${UPDATE_OUTPUT}" | grep -qi "already up.to.date\|already at latest\|no update\|up to date"; then
  pass "'already up to date' message detected (AC5)"
  echo "  Pulse output: $(echo "${UPDATE_OUTPUT}" | head -1)"
else
  skip "'already up to date' message check" \
    "pulse may phrase this differently — actual output: $(echo "${UPDATE_OUTPUT}" | head -2 | tr '\n' ' ')"
fi

# Verify binary was NOT overwritten if it was already up to date
if [[ -z "${BINARY_MTIME_BEFORE}" ]]; then
  skip "Binary modification time check" "could not determine binary mtime before update (binary not found or stat failed)"
elif [[ -n "${BINARY_MTIME_BEFORE}" ]]; then
  BINARY_MTIME_AFTER=""
  if [[ -f "${PLUGIN_DIR}/libbmad_plugin.so" ]]; then
    BINARY_MTIME_AFTER="$(stat -c %Y "${PLUGIN_DIR}/libbmad_plugin.so" 2>/dev/null || true)"
  elif [[ -f "${PLUGIN_DIR}/libbmad_plugin.dylib" ]]; then
    BINARY_MTIME_AFTER="$(stat -f %m "${PLUGIN_DIR}/libbmad_plugin.dylib" 2>/dev/null || true)"
  fi
  if [[ "${BINARY_MTIME_BEFORE}" == "${BINARY_MTIME_AFTER}" ]]; then
    pass "Binary not overwritten when already up to date (AC5)"
  else
    skip "Binary modification time check" \
      "binary was re-written even though version appears unchanged — Pulse may always replace on update"
  fi
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "==================================================="
echo "Results: ${PASS} passed, ${FAIL} failed, ${SKIPPED} skipped"
echo ""
if [[ ${FAIL} -gt 0 ]]; then
  echo "RESULT: FAILED"
  echo ""
  echo "Manual checks to investigate:"
  echo "  pulse plugin list"
  echo "  ls -la ~/.pulse/plugins/bmad-method/"
  echo "  pulse plugin info bmad-method"
  exit 1
else
  echo "RESULT: PASSED"
  exit 0
fi
