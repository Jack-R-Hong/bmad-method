# Story 8.3: Add Tests for suggested_config Wildcard Fallback

Status: ready-for-dev

## Story

As a QA engineer,
I want tests verifying that the `suggested_config_for()` wildcard fallback returns correct default values,
so that agents without explicit match arms (analyst, devops, scrum-master, security, tech-writer, ux-designer, quick-flow, pm) receive the expected default configuration.

## Acceptance Criteria

**Given** the `suggested_config_for()` function in executor.rs has explicit match arms for architect, bmad-master, dev, and qa
**And** a wildcard `_` fallback arm for all other agents
**When** a fallback agent name is passed (e.g., "bmad/analyst")
**Then** the returned config has model_tier="sonnet", max_turns=20, permission_mode="plan", allowed_tools=None

**Given** at least 3 fallback agents are tested (analyst, devops, security)
**When** `cargo test -p bmad-plugin` runs
**Then** all 3 fallback tests pass, confirming the wildcard arm returns the expected defaults

**Given** the wildcard fallback config values change in the future
**When** the tests run
**Then** they fail, alerting developers to the config change so dependent systems can be updated

## Tasks / Subtasks

- [ ] **Task 1: Add `suggested_config_for_analyst_is_fallback_default` test** (AC: fallback values)
  - Add test to `executor.rs` `#[cfg(test)] mod tests`
  - Call `suggested_config_for("bmad/analyst")`
  - Assert `.unwrap()` returns `SuggestedConfig` with:
    - `model_tier` == Some("sonnet")
    - `max_turns` == Some(20)
    - `permission_mode` == Some("plan")
    - `allowed_tools` == None

- [ ] **Task 2: Add `suggested_config_for_devops_is_fallback_default` test** (AC: fallback values)
  - Call `suggested_config_for("bmad/devops")`
  - Assert same default values as Task 1

- [ ] **Task 3: Add `suggested_config_for_security_is_fallback_default` test** (AC: fallback values)
  - Call `suggested_config_for("bmad/security")`
  - Assert same default values as Task 1

- [ ] **Task 4 (optional): Add additional fallback agent tests** (AC: broader coverage)
  - Consider adding tests for scrum-master, tech-writer, ux-designer, quick-flow, pm
  - These are optional but increase confidence that no agent is accidentally matched by a future explicit arm

- [ ] **Task 5: Run and validate** (AC: all pass)
  - Run `cargo test -p bmad-plugin suggested_config` to verify all config tests pass
  - Run `cargo test -p bmad-plugin` to confirm no regressions

## Dev Notes

The `suggested_config_for()` function is defined at executor.rs:33-62. It strips the "bmad/" prefix and matches:

- `"architect" | "bmad-master"` -> opus, 20 turns, plan
- `"dev" | "developer"` -> sonnet, 30 turns, bypassPermissions
- `"qa"` -> sonnet, 15 turns, plan
- `_` (wildcard) -> sonnet, 20 turns, plan, allowed_tools=None

Existing tests (lines 472-499) cover the 4 explicit arms. The new tests target the `_` wildcard arm specifically.

Follow the exact pattern of the existing `suggested_config_for_architect_is_opus` test:

```rust
#[test]
fn suggested_config_for_analyst_is_fallback_default() {
    let cfg = suggested_config_for("bmad/analyst").unwrap();
    assert_eq!(cfg.model_tier.as_deref(), Some("sonnet"));
    assert_eq!(cfg.max_turns, Some(20));
    assert_eq!(cfg.permission_mode.as_deref(), Some("plan"));
    assert!(cfg.allowed_tools.is_none());
}
```

The `suggested_config_for` function is private (no `pub`), but it is accessible within the same file's `#[cfg(test)]` module via `use super::*`.

### Project Structure Notes

- **Target file:** `crates/bmad-plugin/src/executor.rs` (test module at line 134)
- **Function under test:** `suggested_config_for()` at executor.rs:33-62
- **Existing config tests:** executor.rs lines 472-499
- **SuggestedConfig type:** defined in `crates/bmad-types/src/config.rs`

### References

- Epic 8 (Test Coverage Completeness) from v2 improvements epic file
- `suggested_config_for()` wildcard arm at executor.rs:54-59

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
