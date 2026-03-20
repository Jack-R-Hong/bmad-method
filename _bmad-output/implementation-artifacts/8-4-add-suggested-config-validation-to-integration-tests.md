# Story 8.4: Add suggested_config Validation to Integration Tests

Status: ready-for-dev

## Story

As a QA engineer,
I want the integration tests to validate `suggested_config` in agent output,
so that the full plugin execution path (including config generation) is verified end-to-end, not just unit-tested in isolation.

## Acceptance Criteria

**Given** the three-agent sequential workflow test (`test_three_agent_sequential_workflow`) in `tests/plugin_integration.rs`
**When** it validates architect, dev, and qa outputs
**Then** it also asserts that `suggested_config` is non-null for each agent
**And** `suggested_config.model_tier` and `suggested_config.max_turns` are present (non-null)

**Given** the architect output in the sequential test
**When** `suggested_config` is checked
**Then** `model_tier` == "opus" and `permission_mode` == "plan"

**Given** the dev output in the sequential test
**When** `suggested_config` is checked
**Then** `model_tier` == "sonnet" and `permission_mode` == "bypassPermissions"

**Given** the parallel agents test (`test_parallel_agents_no_shared_state`)
**When** architect and qa execute in parallel threads
**Then** both outputs include non-null `suggested_config` with `model_tier` and `max_turns` present

**Given** all integration tests run
**When** `cargo test --test plugin_integration` executes
**Then** all existing and new assertions pass with no regressions

## Tasks / Subtasks

- [ ] **Task 1: Add suggested_config assertions to `test_three_agent_sequential_workflow`** (AC: non-null, architect=opus, dev=bypassPermissions)
  - After the existing architect output assertions (around line 56), add:
    - `assert!(arch_out["suggested_config"].is_object(), "architect suggested_config must be present")`
    - `assert_eq!(arch_out["suggested_config"]["model_tier"].as_str(), Some("opus"))`
    - `assert_eq!(arch_out["suggested_config"]["permission_mode"].as_str(), Some("plan"))`
    - `assert!(arch_out["suggested_config"]["max_turns"].as_u64().is_some())`
  - After the existing dev output assertions (around line 74), add:
    - `assert!(dev_out["suggested_config"].is_object(), "dev suggested_config must be present")`
    - `assert_eq!(dev_out["suggested_config"]["model_tier"].as_str(), Some("sonnet"))`
    - `assert_eq!(dev_out["suggested_config"]["permission_mode"].as_str(), Some("bypassPermissions"))`
    - `assert_eq!(dev_out["suggested_config"]["max_turns"].as_u64(), Some(30))`
  - After the existing qa output assertions (around line 92), add:
    - `assert!(qa_out["suggested_config"].is_object(), "qa suggested_config must be present")`
    - `assert_eq!(qa_out["suggested_config"]["model_tier"].as_str(), Some("sonnet"))`
    - `assert_eq!(qa_out["suggested_config"]["max_turns"].as_u64(), Some(15))`

- [ ] **Task 2: Add suggested_config assertions to `test_parallel_agents_no_shared_state`** (AC: parallel agents)
  - After the existing parallel test assertions (around line 160), add:
    - `assert!(arch_out["suggested_config"].is_object(), "parallel architect suggested_config must be present")`
    - `assert!(arch_out["suggested_config"]["model_tier"].as_str().is_some())`
    - `assert!(arch_out["suggested_config"]["max_turns"].as_u64().is_some())`
    - `assert!(qa_out["suggested_config"].is_object(), "parallel qa suggested_config must be present")`
    - `assert!(qa_out["suggested_config"]["model_tier"].as_str().is_some())`
    - `assert!(qa_out["suggested_config"]["max_turns"].as_u64().is_some())`

- [ ] **Task 3: Run and validate** (AC: all pass)
  - Run `cargo test --test plugin_integration` and verify all tests pass
  - Run `cargo test` (full workspace) to confirm no regressions

## Dev Notes

The integration tests in `tests/plugin_integration.rs` use `serde_json::Value` for output parsing (via the `parse_output` helper at line 37). The `suggested_config` field is serialized as a JSON object by `BmadOutput` (executor.rs:29), so it can be accessed via `out["suggested_config"]["model_tier"]` etc.

The `BmadOutput` struct (executor.rs:24-31) has `suggested_config: Option<SuggestedConfig>`. Since `suggested_config_for()` always returns `Some(...)` (executor.rs:61), the field will always be present in serialized output. However, the integration tests should still validate this explicitly to catch any future refactoring that might change this behavior.

Key values to assert per agent (from executor.rs:35-61):
- **architect:** model_tier="opus", max_turns=20, permission_mode="plan"
- **dev:** model_tier="sonnet", max_turns=30, permission_mode="bypassPermissions"
- **qa:** model_tier="sonnet", max_turns=15, permission_mode="plan"

The parallel test currently only runs architect and qa. Both should have their suggested_config validated but specific value assertions are optional for the parallel test -- the primary goal is confirming config presence under concurrent execution.

### Project Structure Notes

- **Target file:** `tests/plugin_integration.rs`
- **Sequential workflow test:** `test_three_agent_sequential_workflow` at line 42
- **Parallel agents test:** `test_parallel_agents_no_shared_state` at line 118
- **Output parse helper:** `parse_output()` at line 37 (returns `serde_json::Value`)
- **BmadOutput struct:** `crates/bmad-plugin/src/executor.rs` lines 24-31
- **suggested_config_for():** `crates/bmad-plugin/src/executor.rs` lines 33-62

### References

- Epic 8 (Test Coverage Completeness) from v2 improvements epic file
- Integration test file: `tests/plugin_integration.rs`
- Config values defined in `suggested_config_for()` at executor.rs:35-61

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
