# Story 7.2: Make `prompt` Fallback Explicit with Logging

Status: done

## Story

As a plugin consumer,
I want to know when the prompt field is missing and the system falls back to `task.description`,
so that I can debug unexpected output without reading plugin source code.

## Acceptance Criteria

**Given** a `BmadInput` with `prompt: None` (e.g., `{"agent": "bmad/architect"}`)
**When** `extract_user_context()` falls back to `task.description`
**Then** a `tracing::warn!` log is emitted: `"prompt field missing in BmadInput, falling back to task.description"`

**Given** a `BmadInput` with `prompt: Some("...")`
**When** `extract_user_context()` processes it
**Then** no warning is logged

**Given** a raw string input (not valid `BmadInput` JSON)
**When** `extract_user_context()` falls back to using the raw string
**Then** no warning is logged (this is the non-JSON path, not a missing-field situation)

**Given** `task.input` is `None` and `extract_user_context()` falls back to `task.description`
**When** the fallback occurs
**Then** a `tracing::warn!` log is emitted: `"task.input is None, falling back to task.description"`

**Given** the fallback behavior is implemented
**When** a developer reads the `BmadInput` struct definition
**Then** a doc comment on the `prompt` field states: `/// Optional user prompt. If omitted, falls back to task.description.`

**Given** the fallback contract needs documentation
**When** `docs/pulse-api-contract.md` is reviewed
**Then** it documents: "If the `prompt` field is omitted from `BmadInput`, the plugin uses `task.description` as user context. A `tracing::warn!` log is emitted when this fallback occurs."

## Tasks / Subtasks

- [ ] **Task 1: Add `tracing` dependency if not already present** (AC: #1)
  - [ ] Check `crates/bmad-plugin/Cargo.toml` for `tracing` dependency
  - [ ] If missing, add `tracing = "0.1"` to `[dependencies]`
  - [ ] Add `use tracing;` or `use tracing::warn;` import in `executor.rs`

- [ ] **Task 2: Add warning log on `prompt: None` fallback** (AC: #1, #2)
  - [ ] In `extract_user_context()` at line 119 where `bmad.prompt.unwrap_or_else(|| task.description.clone())` executes, refactor to an explicit match:
    ```rust
    let text = match bmad.prompt {
        Some(prompt) => prompt,
        None => {
            tracing::warn!("prompt field missing in BmadInput, falling back to task.description");
            task.description.clone()
        }
    };
    ```
  - [ ] Ensure the warn log fires only when `prompt` is `None`, not when the JSON parse fails (the `else` branch at line 120-122 handles non-JSON input)

- [ ] **Task 3: Add warning log on `task.input` is `None` fallback** (AC: #4)
  - [ ] In the `else` branch (line 123-125) where `task.input` is `None`:
    ```rust
    } else {
        tracing::warn!("task.input is None, falling back to task.description");
        task.description.clone()
    };
    ```

- [ ] **Task 4: Add doc comment on `BmadInput.prompt` field** (AC: #5)
  - [ ] Add `/// Optional user prompt. If omitted, falls back to task.description.` above the `prompt` field in the `BmadInput` struct (executor.rs line 12-13)

- [ ] **Task 5: Document fallback contract in API docs** (AC: #6)
  - [ ] Add a "Prompt Fallback Behavior" subsection to `docs/pulse-api-contract.md`
  - [ ] Document: when fallback triggers, what log is emitted, and the explicit contract that `task.description` is used when `prompt` is absent

- [ ] **Task 6: Write tests** (AC: #1, #2, #3)
  - [ ] Test: `BmadInput` with `prompt: Some(...)` produces output without triggering the fallback path — verify the user_context matches the prompt value
  - [ ] Test: `BmadInput` with `prompt: None` (JSON: `{"agent": "bmad/test-agent"}`) uses `task.description` as user_context — verify the output `user_context` equals the task description
  - [ ] Test: Raw non-JSON input string goes through the non-JSON path and is used verbatim
  - [ ] Note: Testing that `tracing::warn!` was actually emitted requires `tracing-test` or similar. If adding that dependency is too heavy, rely on code review of the log placement and test the functional behavior (correct `user_context` value) instead.

## Dev Notes

### Current Code to Refactor

In `crates/bmad-plugin/src/executor.rs` lines 116-132:

```rust
fn extract_user_context(task: &TaskInput) -> Result<String, WitPluginError> {
    let text = if let Some(input_str) = task.input.as_deref() {
        if let Ok(bmad) = serde_json::from_str::<BmadInput>(input_str) {
            bmad.prompt.unwrap_or_else(|| task.description.clone())  // <-- silent fallback
        } else {
            input_str.to_string()
        }
    } else {
        task.description.clone()  // <-- silent fallback
    };
    // ...
}
```

The `unwrap_or_else` at line 119 is the primary target. Replace with an explicit `match` that logs before falling back.

### Tracing in WASM Context

If this plugin runs as a WASM component, verify that `tracing` output is captured by the host runtime. In most Pulse plugin SDKs, `tracing` events are forwarded to the host's log collector. If `tracing` is not available in the WASM target, use `eprintln!` as a fallback or the plugin SDK's logging facility. Check `pulse_plugin_sdk` for a logging API.

### Decision: `used_description_fallback` Field vs Documentation

The epic offers two options: add a `used_description_fallback: bool` field to `BmadOutput`, or document the contract. The documentation-only approach is recommended for this story because:
- Adding a field to `BmadOutput` changes the serialized JSON schema, requiring consumer updates
- The `tracing::warn!` log provides runtime visibility
- The doc comment and API contract doc provide developer visibility
- If the field is later desired, it can be added in a separate story with schema versioning (Epic 9)

### Project Structure Notes

```
crates/bmad-plugin/
  Cargo.toml          <- Verify/add tracing dependency
  src/executor.rs     <- Refactor extract_user_context() fallback, add doc comment on BmadInput.prompt

docs/
  pulse-api-contract.md  <- Add "Prompt Fallback Behavior" section
```

### References

- `_bmad-output/planning-artifacts/epics-v2-improvements.md` lines 340-367: Story 7.2 epic definition
- `crates/bmad-plugin/src/executor.rs` lines 116-132: current `extract_user_context()` with silent fallback
- `crates/bmad-plugin/src/executor.rs` lines 8-14: `BmadInput` struct definition
- Deficiency D7 in the v2 improvements epic: "prompt field silently falls back to task.description"

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
