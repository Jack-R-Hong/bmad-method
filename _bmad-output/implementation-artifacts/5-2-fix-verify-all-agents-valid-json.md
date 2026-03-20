# Story 5.2: Fix verify_all_agents to Send Valid JSON Through Full Execute Path

Status: ready-for-dev

## Story

As a plugin maintainer,
I want verify_all_agents() health checks to send valid BmadInput JSON through the full deserialization and execution path,
so that health checks actually verify the real code path and catch serialization bugs before deployment.

## Acceptance Criteria

**Given** `verify_all_agents()` is called
**When** each agent's health check executes
**Then** the TaskInput contains valid BmadInput JSON with the correct `agent` field matching the agent's `executor_name` and a non-empty `prompt` field

**Given** a health check completes successfully for an agent
**When** the output is inspected
**Then** the output contains a non-empty `system_prompt` and the `agent` field matches the executor name used in the request

**Given** a health check fails for an agent
**When** the failure result is returned
**Then** `failure_reason` contains an actionable message indicating what went wrong (not just a generic error)

**Given** all changes are complete
**When** `cargo test --workspace` is run
**Then** all existing tests pass plus new tests verify the health check sends valid JSON

## Tasks / Subtasks

- [ ] **Task 1: Fix health check input in verify_all_agents()** (AC: #1)
  - [ ] In `crates/bmad-plugin/src/registry.rs:29`, replace `TaskInput::new("health-check", "ping").with_input("ping")` with a valid BmadInput JSON payload
  - [ ] Construct JSON: `{"agent": "<executor_name>", "prompt": "health-check"}` where `<executor_name>` is `meta.executor_name`
  - [ ] Use `serde_json::json!` macro or `format!` to build the JSON string
  - [ ] Example replacement:
    ```rust
    let input_json = serde_json::json!({
        "agent": meta.executor_name,
        "prompt": "health-check"
    }).to_string();
    let task = TaskInput::new("health-check", "health-check").with_input(&input_json);
    ```

- [ ] **Task 2: Validate output content in health check** (AC: #2)
  - [ ] After `executor.execute(task, config)` returns `Ok(result)`, deserialize `result.content` as JSON
  - [ ] Verify that the output contains a non-empty `system_prompt` field
  - [ ] Verify that the output `agent` field matches `meta.executor_name`
  - [ ] If validation fails, return `VerificationResult { passed: false, failure_reason: Some("...") }`
  - [ ] Use `crate::executor::BmadOutput` for deserialization (it is already defined in executor.rs)

- [ ] **Task 3: Improve failure reasons** (AC: #3)
  - [ ] On execution error: include the error message, e.g., `format!("execute failed: {e}")`
  - [ ] On output validation failure: include which field was invalid, e.g., `"system_prompt is empty"` or `"agent field mismatch: expected 'bmad/architect', got 'bmad/dev'"`
  - [ ] On missing content: `"output content is None"`
  - [ ] On JSON parse failure: `format!("output is not valid JSON: {e}")`

- [ ] **Task 4: Add serde_json import if not already present** (AC: #1)
  - [ ] Check if `serde_json` is already in scope in `registry.rs` — it is likely not imported at module level
  - [ ] Add `use serde_json;` or inline usage as needed
  - [ ] Verify `serde_json` is already a dependency in `crates/bmad-plugin/Cargo.toml` (it is, used by executor.rs)

- [ ] **Task 5: Update existing tests** (AC: #4)
  - [ ] The test `verify_all_agents_all_pass_with_valid_input` in registry.rs:302-305 should still pass since valid JSON is now sent
  - [ ] Add a new test that inspects the actual output content of a health-checked agent to verify system_prompt is non-empty
  - [ ] Add a test that verifies the health check JSON input is valid BmadInput (can deserialize the constructed JSON)

- [ ] **Task 6: Verify clean build** (AC: #4)
  - [ ] Run `cargo build --workspace` — zero warnings
  - [ ] Run `cargo test --workspace` — all tests pass
  - [ ] Run `cargo clippy --workspace` — no new lints

## Dev Notes

### Architecture Context

The current `verify_all_agents()` function in `crates/bmad-plugin/src/registry.rs:23-45` sends a bare string `"ping"` as the task input:

```rust
let task = TaskInput::new("health-check", "ping").with_input("ping");
```

This bypasses the JSON deserialization path in `crates/bmad-plugin/src/lib.rs:51`:
```rust
let bmad_input: BmadInput = serde_json::from_str(input_str)
    .map_err(|e| WitPluginError::invalid_input(format!("invalid BMAD input JSON: {e}")))?;
```

However, note that `verify_all_agents()` calls `executor.execute()` directly (the `BmadExecutor::execute` method), NOT `BmadMethodPlugin::execute`. The `BmadExecutor::execute` method in executor.rs:87-113 does NOT parse BmadInput — it calls `extract_user_context(&task)` which tries to parse BmadInput but falls back to using the raw string (executor.rs:117-131). So currently the "ping" string works but bypasses JSON validation.

The fix should ensure the health check sends proper BmadInput JSON so that `extract_user_context` actually exercises the BmadInput deserialization branch (executor.rs:118).

### BmadOutput Structure (for validation in Task 2)

The output is serialized as `BmadOutput` in `crates/bmad-plugin/src/executor.rs:24-31`:
```rust
pub struct BmadOutput {
    pub agent: String,
    pub system_prompt: String,
    pub user_context: String,
    pub suggested_params: Option<GenerationParams>,
    pub suggested_config: Option<SuggestedConfig>,
    pub metadata: BmadOutputMetadata,
}
```

To deserialize the output in registry.rs, you can use `crate::executor::BmadOutput` — it derives both `Serialize` and `Deserialize`.

### Key Files to Modify

| File | Action |
|------|--------|
| `crates/bmad-plugin/src/registry.rs` | Fix `verify_all_agents()` lines 29-44 |

### Important Constraints

- Never panic in plugin code — all validation must return `Result` or use `VerificationResult`
- `verify_all_agents()` returns `Vec<VerificationResult>`, not `Result` — individual failures are captured per-agent
- The function must remain self-contained (no external dependencies beyond what's in the crate)
- Keep the existing function signature: `pub fn verify_all_agents() -> Vec<VerificationResult>`

### Project Structure Notes

```
crates/bmad-plugin/src/
├── registry.rs      ← modify verify_all_agents() (lines 23-45)
├── executor.rs      ← BmadOutput struct (derive Deserialize) for validation
├── lib.rs           ← NOT modified (plugin-level execute is separate)
└── generated/       ← NOT modified
```

### References

- `crates/bmad-plugin/src/registry.rs:23-45` — the function to fix
- `crates/bmad-plugin/src/executor.rs:8-14` — BmadInput struct (the JSON format to send)
- `crates/bmad-plugin/src/executor.rs:16-31` — BmadOutput struct (for validating health check output)
- `crates/bmad-plugin/src/executor.rs:116-131` — extract_user_context fallback behavior
- `crates/bmad-plugin/src/lib.rs:44-72` — the plugin-level execute showing proper BmadInput usage

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
