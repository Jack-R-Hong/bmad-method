# Story 6.3: Remove Hardcoded suggested_config_for() Match Block

Status: done

## Story

As a plugin maintainer,
I want to remove the hardcoded `suggested_config_for()` function from `executor.rs`,
so that all agent configuration is driven entirely from frontmatter data through code generation, eliminating manual maintenance of agent-specific values in Rust source.

## Acceptance Criteria

**AC1: `suggested_config_for()` function is deleted**
**Given** the executor module
**When** a developer inspects `crates/bmad-plugin/src/executor.rs`
**Then** the `suggested_config_for()` function (previously lines 33-62) no longer exists
**And** there are no hardcoded agent names (e.g., "architect", "bmad-master", "dev", "qa") anywhere in `executor.rs`

**AC2: BmadExecutor receives config exclusively from constructor**
**Given** `BmadExecutor::for_agent()` is called with a `suggested_config: Option<SuggestedConfig>` parameter (added in 6.2)
**When** `execute()` produces `BmadOutput`
**Then** `suggested_config` is taken from `self.suggested_config` (the constructor parameter)
**And** no fallback lookup or match block is consulted

**AC3: All agent outputs preserve identical config values**
**Given** the plugin is built with generated config from 6.2
**When** each of the 12 agents is executed
**Then** the `suggested_config` in the output JSON matches the values from frontmatter:
- architect: `{"model_tier": "opus", "max_turns": 20, "permission_mode": "plan"}`
- bmad-master: `{"model_tier": "opus", "max_turns": 20, "permission_mode": "plan"}`
- developer: `{"model_tier": "sonnet", "max_turns": 30, "permission_mode": "bypassPermissions"}`
- qa: `{"model_tier": "sonnet", "max_turns": 15, "permission_mode": "plan"}`
- All others: `{"model_tier": "sonnet", "max_turns": 20, "permission_mode": "plan"}`

**AC4: Tests are migrated from hardcoded function to generated values**
**Given** the existing tests `suggested_config_for_architect_is_opus`, `suggested_config_for_bmad_master_is_opus`, `suggested_config_for_dev_is_sonnet_bypass`, `suggested_config_for_qa_is_sonnet_plan`
**When** tests are updated
**Then** they call `generated::architect::suggested_config()` (etc.) instead of the deleted `suggested_config_for()` function
**And** all assertions remain identical (same model_tier, max_turns, permission_mode values)

**AC5: No hardcoded agent names remain in executor.rs**
**Given** the cleaned executor module
**When** a developer searches for string literals in `executor.rs`
**Then** no agent-specific strings like `"architect"`, `"bmad-master"`, `"dev"`, `"qa"` appear
**And** the only string literals are generic ones like `"bmad-method"`, `"output serialization failed"`, `"input cannot be empty"`

**AC6: All tests pass across the workspace**
**Given** the removal is complete
**When** `cargo test --workspace` is run
**Then** all tests pass in `bmad-converter`, `bmad-plugin`, and `bmad-types`

## Tasks / Subtasks

- [ ] **Task 1: Delete `suggested_config_for()` function** (AC: #1, #5)
  - [ ] In `crates/bmad-plugin/src/executor.rs`, delete the entire `suggested_config_for()` function (lines 33-62)
  - [ ] Verify no other code in the file references `suggested_config_for` (it should already be unused after Story 6.2 wired `self.suggested_config` into `execute()`)

- [ ] **Task 2: Verify `execute()` uses only `self.suggested_config`** (AC: #2)
  - [ ] Confirm that `execute()` line 100 uses `self.suggested_config.clone()` (set in Story 6.2), not `suggested_config_for(self.metadata.executor_name)`
  - [ ] If Story 6.2 left a transitional call to `suggested_config_for()`, replace it now with `self.suggested_config.clone()`

- [ ] **Task 3: Migrate config tests to use generated functions** (AC: #4)
  - [ ] Replace test `suggested_config_for_architect_is_opus`:
    ```rust
    #[test]
    fn architect_suggested_config_is_opus() {
        let cfg = generated::architect::suggested_config().unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("opus"));
        assert_eq!(cfg.max_turns, Some(20));
        assert_eq!(cfg.permission_mode.as_deref(), Some("plan"));
    }
    ```
  - [ ] Replace test `suggested_config_for_bmad_master_is_opus`:
    ```rust
    #[test]
    fn bmad_master_suggested_config_is_opus() {
        let cfg = generated::bmad_master::suggested_config().unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("opus"));
        assert_eq!(cfg.max_turns, Some(20));
        assert_eq!(cfg.permission_mode.as_deref(), Some("plan"));
    }
    ```
  - [ ] Replace test `suggested_config_for_dev_is_sonnet_bypass`:
    ```rust
    #[test]
    fn developer_suggested_config_is_sonnet_bypass() {
        let cfg = generated::developer::suggested_config().unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("sonnet"));
        assert_eq!(cfg.permission_mode.as_deref(), Some("bypassPermissions"));
        assert_eq!(cfg.max_turns, Some(30));
    }
    ```
  - [ ] Replace test `suggested_config_for_qa_is_sonnet_plan`:
    ```rust
    #[test]
    fn qa_suggested_config_is_sonnet_plan() {
        let cfg = generated::qa::suggested_config().unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("sonnet"));
        assert_eq!(cfg.max_turns, Some(15));
        assert_eq!(cfg.permission_mode.as_deref(), Some("plan"));
    }
    ```

- [ ] **Task 4: Add comprehensive config test for all 12 agents** (AC: #3, #6)
  - [ ] Add a test that iterates over `generated::all_agent_entries()` and asserts every agent has `Some(SuggestedConfig)`:
    ```rust
    #[test]
    fn all_agents_have_suggested_config() {
        let entries = generated::all_agent_entries();
        for (meta, _, _, config) in &entries {
            assert!(
                config.is_some(),
                "agent {} must have suggested_config after 6.1 frontmatter update",
                meta.executor_name
            );
        }
    }
    ```

- [ ] **Task 5: Update `BmadExecutor::for_agent` calls in test helpers** (AC: #4, #6)
  - [ ] The `TEST_META`-based tests that use `BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None)` need a 4th argument `None` for `suggested_config` (should already be done in 6.2, verify here)
  - [ ] Integration tests using `generated::architect::*` should pass `generated::architect::suggested_config()` as the 4th argument

- [ ] **Task 6: Audit executor.rs for remaining hardcoded agent references** (AC: #5)
  - [ ] Search `executor.rs` for any remaining string matching agent names
  - [ ] Verify only generic strings remain: `"bmad-method"`, `"output serialization failed"`, `"input cannot be empty"`, `"bmad/"` (in prefix-stripping, if still present — remove if unused)
  - [ ] If the `trim_start_matches("bmad/")` line from the old function remains, delete it

- [ ] **Task 7: Run full workspace tests** (AC: #6)
  - [ ] Run `cargo test --workspace`
  - [ ] Run `cargo clippy --workspace` to catch dead code warnings (the deleted function should not produce warnings if fully removed)

## Dev Notes

### Architecture Patterns
- This is a pure cleanup story: delete code and migrate tests
- After this story, the data flow is: `agents/*.md` -> `bmad-converter` -> `generated/*.rs` (with `suggested_config()`) -> `BmadExecutor` constructor -> `BmadOutput`
- No runtime lookup or match logic remains — config is baked in at compile time
- Error handling: no new error paths. Plugin code continues to return `Result` everywhere

### Risk: Story 6.2 Transitional State
Story 6.2 may have left `suggested_config_for()` in place but stopped calling it (to keep tests passing during the transition). This story completes the cleanup. If 6.2 did NOT stop calling it (i.e., `execute()` still calls `suggested_config_for()`), Task 2 must also update `execute()` to use `self.suggested_config.clone()`.

### Test Migration Pattern
The test names are renamed from `suggested_config_for_X_is_Y` (testing a standalone function) to `X_suggested_config_is_Y` (testing the generated module function). This makes it clear the tests are validating generated output, not a runtime lookup.

### Project Structure Notes

| Path | Role | Changes |
|------|------|---------|
| `crates/bmad-plugin/src/executor.rs` | Runtime executor | Delete `suggested_config_for()` (lines 33-62), migrate 4 tests, add 1 new test |
| `crates/bmad-plugin/src/lib.rs` | Plugin entry point | No changes (already updated in 6.2) |
| `crates/bmad-plugin/src/generated/*.rs` | Generated modules | No changes (already contain `suggested_config()` from 6.2) |

### References
- Depends on: Story 6.2 (generated `suggested_config()` functions and 4-tuple `all_agent_entries()` must exist)
- Function to delete: `crates/bmad-plugin/src/executor.rs` lines 33-62 (`suggested_config_for()`)
- Tests to migrate: `crates/bmad-plugin/src/executor.rs` lines 472-499 (4 tests)
- Constructor updated in 6.2: `crates/bmad-plugin/src/executor.rs` lines 71-81 (`BmadExecutor::for_agent()`)
- Dispatch updated in 6.2: `crates/bmad-plugin/src/lib.rs` lines 54-70 (4-tuple destructuring)

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
