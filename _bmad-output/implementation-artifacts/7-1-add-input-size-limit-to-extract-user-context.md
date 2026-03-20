# Story 7.1: Add Input Size Limit to `extract_user_context`

Status: ready-for-dev

## Story

As a plugin security reviewer,
I want prompt input to be bounded by a maximum size,
so that oversized payloads cannot cause excessive memory allocation, especially in WASM environments.

## Acceptance Criteria

**Given** a `MAX_INPUT_LEN` constant is defined as 128KB (131072 bytes)
**When** a developer reviews `executor.rs`
**Then** `MAX_INPUT_LEN` is documented with a comment explaining the rationale (WASM memory constraints, output size control)

**Given** `extract_user_context()` receives input exceeding `MAX_INPUT_LEN`
**When** the length check runs
**Then** it returns `Err(WitPluginError::invalid_input("input exceeds maximum length of 131072 bytes"))`

**Given** input is exactly at the limit (131072 bytes)
**When** `extract_user_context()` processes it
**Then** it succeeds without error

**Given** input is 1 byte over the limit (131073 bytes)
**When** `extract_user_context()` processes it
**Then** it returns the size limit error

**Given** the size check is implemented
**When** `cargo test -p bmad-plugin` runs
**Then** new tests verify: at-limit input passes, over-limit input fails with descriptive error, and the error message contains the limit value `131072`

## Tasks / Subtasks

- [ ] **Task 1: Define `MAX_INPUT_LEN` constant** (AC: #1)
  - [ ] Add `const MAX_INPUT_LEN: usize = 131_072;` at module level in `crates/bmad-plugin/src/executor.rs`
  - [ ] Add doc comment: `/// Maximum allowed input size in bytes. Prevents oversized payloads from causing excessive memory allocation in WASM environments and controls output JSON size.`
  - [ ] Place the constant near the top of the file, after imports and before struct definitions

- [ ] **Task 2: Add size validation to `extract_user_context()`** (AC: #2, #3, #4)
  - [ ] In `extract_user_context()`, after the `text` variable is resolved (after line 125 in the current code) but before the empty check, add the size check
  - [ ] Check `text.len() > MAX_INPUT_LEN` — use byte length, not char count, for consistency with memory allocation concerns
  - [ ] Return `Err(WitPluginError::invalid_input(format!("input exceeds maximum length of {MAX_INPUT_LEN} bytes")))` on failure
  - [ ] Ensure the size check runs before the empty check so the error messages do not conflict (an oversized empty-looking input is an edge case not worth optimizing)

- [ ] **Task 3: Write unit tests** (AC: #3, #4, #5)
  - [ ] `input_at_max_length_succeeds`: Create a string of exactly 131072 bytes, verify `extract_user_context()` returns `Ok`
  - [ ] `input_over_max_length_returns_error`: Create a string of 131073 bytes, verify it returns `Err` with code `"invalid_input"`
  - [ ] `input_over_max_length_error_contains_limit`: Verify the error message contains `"131072"`
  - [ ] Use `"a".repeat(N)` to construct test inputs — simple and deterministic
  - [ ] Tests must call `extract_user_context()` directly (it is `fn`, not a method on `BmadExecutor`) by constructing a `TaskInput` with the oversized string as input

## Dev Notes

### Implementation Location

The size check goes into `extract_user_context()` in `crates/bmad-plugin/src/executor.rs` (currently lines 116-132). The function signature is:

```rust
fn extract_user_context(task: &TaskInput) -> Result<String, WitPluginError>
```

The check should be inserted after `text` is fully resolved (line 125) and before the existing empty check (line 127-129):

```rust
// After text is resolved:
if text.len() > MAX_INPUT_LEN {
    return Err(WitPluginError::invalid_input(
        format!("input exceeds maximum length of {MAX_INPUT_LEN} bytes"),
    ));
}

if text.trim().is_empty() {
    return Err(WitPluginError::invalid_input("input cannot be empty"));
}
```

### Error Type

Use `WitPluginError::invalid_input()` — the same error constructor used for the existing empty-input check. This is the plugin boundary error type from `pulse_plugin_sdk::error::WitPluginError`. Do NOT use `BmadError` here; `extract_user_context` already returns `WitPluginError` directly.

### Byte Length vs Char Count

Use `.len()` (byte count) rather than `.chars().count()` (Unicode scalar count). Rationale:
- Memory allocation is proportional to byte length, not character count
- `.len()` is O(1) on `String`; `.chars().count()` is O(n)
- WASM linear memory limits are byte-based

### Test Pattern

Tests in this file use `TaskInput::new()` and `TaskInput::with_input()` to construct test inputs. Follow the existing pattern:

```rust
fn test_task(prompt: &str) -> TaskInput {
    TaskInput::new("t-1", prompt).with_input(
        &serde_json::json!({
            "agent": "bmad/test-agent",
            "prompt": prompt
        })
        .to_string(),
    )
}
```

For the size limit tests, the `prompt` field in the JSON will itself be large. Ensure the JSON wrapper overhead does not push the total `input` string over the limit prematurely — the limit applies to the resolved `text` (the prompt value extracted from JSON), not to the raw `task.input` string.

### Never Panic

Per project conventions: never panic in plugin code. The size check must return `Result`, not `assert!` or `unwrap()`.

### Project Structure Notes

```
crates/bmad-plugin/src/
  executor.rs       <- Add MAX_INPUT_LEN constant and size check in extract_user_context()
```

### References

- `_bmad-output/planning-artifacts/epics-v2-improvements.md` lines 307-337: Story 7.1 epic definition
- `crates/bmad-plugin/src/executor.rs` lines 116-132: current `extract_user_context()` implementation
- Deficiency D6 in the v2 improvements epic: "No input size validation on prompt text"

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
