# Story 7.4: Document Input Sanitization Responsibility Boundary

Status: done

## Story

As a Pulse platform developer,
I want a clear contract about who is responsible for sanitizing prompt content,
so that security-sensitive consumers know they must sanitize before rendering `user_context` in a UI.

## Acceptance Criteria

**Given** `docs/pulse-api-contract.md`
**When** the output section is reviewed
**Then** it contains: "The plugin passes `user_context` through verbatim. It does NOT sanitize, escape, or filter input content. Consumers that render `user_context` or `system_prompt` in HTML or other injection-sensitive contexts MUST sanitize the content before rendering."

**Given** the `BmadOutput` struct definition in `crates/bmad-plugin/src/executor.rs`
**When** a developer reads the doc comment on `user_context`
**Then** it states: "Raw task input. Not sanitized — consumer must sanitize before rendering in injection-sensitive contexts."

**Given** the `BmadOutput` struct definition in `crates/bmad-plugin/src/executor.rs`
**When** a developer reads the doc comment on `system_prompt`
**Then** it states that the system prompt is static embedded content from agent markdown files, but consumers should still treat it as untrusted if the agent markdown source is user-editable

**Given** the `AgentOutput` struct in `crates/bmad-types/src/output.rs`
**When** a developer reads the doc comments
**Then** the `user_context` field doc comment also includes the sanitization warning, consistent with the `BmadOutput` definition

## Tasks / Subtasks

- [ ] **Task 1: Add sanitization contract to `docs/pulse-api-contract.md`** (AC: #1)
  - [ ] Add an "Input Sanitization Responsibility" subsection to the API contract document
  - [ ] Include the following content:
    - The plugin passes `user_context` through verbatim from the workflow input
    - The plugin does NOT sanitize, escape, HTML-encode, or filter any input content
    - The `system_prompt` is static content embedded at compile time from agent `.md` files
    - Consumers rendering `user_context` or `system_prompt` in HTML, terminal UIs, or other injection-sensitive contexts MUST sanitize the content themselves
    - The responsibility boundary: the plugin is a prompt construction layer, not a security layer
    - Common injection risks if content is rendered unsanitized: XSS (HTML), ANSI escape sequences (terminals), prompt injection (LLM context)

- [ ] **Task 2: Add doc comment on `BmadOutput.user_context` field** (AC: #2)
  - [ ] In `crates/bmad-plugin/src/executor.rs`, add a doc comment to the `user_context` field of `BmadOutput`:
    ```rust
    /// Raw task input passed through verbatim. Not sanitized — consumer must
    /// sanitize before rendering in injection-sensitive contexts (HTML, terminals).
    pub user_context: String,
    ```

- [ ] **Task 3: Add doc comment on `BmadOutput.system_prompt` field** (AC: #3)
  - [ ] In `crates/bmad-plugin/src/executor.rs`, add a doc comment to the `system_prompt` field of `BmadOutput`:
    ```rust
    /// Agent persona/role instructions, embedded at compile time from agent .md files.
    /// Content is static but consumers should treat as untrusted if agent markdown
    /// sources are user-editable. Sanitize before rendering in injection-sensitive contexts.
    pub system_prompt: String,
    ```

- [ ] **Task 4: Update doc comment on `AgentOutput.user_context` in bmad-types** (AC: #4)
  - [ ] In `crates/bmad-types/src/output.rs`, update the existing doc comment on `user_context` (currently: "Task input as passed by the Pulse workflow, forwarded to user turn") to include the sanitization warning:
    ```rust
    /// Task input as passed by the Pulse workflow, forwarded to user turn.
    /// Not sanitized — consumer must sanitize before rendering in injection-sensitive contexts.
    pub user_context: String,
    ```

- [ ] **Task 5: Add doc comment on `BmadOutput.metadata` fields for completeness** (AC: #1)
  - [ ] Add brief doc comments on `BmadOutputMetadata` fields in `executor.rs` noting that `persona`, `plugin_name`, and `plugin_version` are static/controlled values (not user-supplied)

## Dev Notes

### Current State

The `user_context` field flows through the following path with zero transformation:

```
Workflow input (task.input / task.description)
  -> extract_user_context()       // only checks: empty, size (after Story 7.1)
    -> BmadOutput.user_context    // verbatim assignment
      -> serde_json::to_string()  // serialized to JSON
        -> StepResult.content     // returned to Pulse
```

At no point is the content sanitized, escaped, or filtered. This is by design: the plugin is a prompt construction layer. But the contract must be explicit so consumers know they bear the sanitization responsibility.

### Two `user_context` Fields

There are two structs with a `user_context` field:
1. `BmadOutput` in `crates/bmad-plugin/src/executor.rs` — the serialized output struct sent to Pulse
2. `AgentOutput` in `crates/bmad-types/src/output.rs` — the shared types crate struct

Both need the sanitization doc comment for consistency. A developer reading either location should see the warning.

### Scope Clarification

This story is documentation-only. No runtime behavior changes. No sanitization logic is being added. The explicit decision is: the plugin does NOT sanitize, and this is documented as the contract.

### Why Not Sanitize in the Plugin

Adding sanitization in the plugin would be harmful because:
- The plugin does not know the rendering context (HTML vs terminal vs raw LLM input)
- HTML-escaping content destined for an LLM prompt would corrupt the prompt
- Double-escaping (plugin + consumer) creates garbled output
- The correct sanitization depends entirely on the consumer's rendering layer

### No Tests Required

This story is pure documentation (doc comments and markdown). No runtime behavior changes. No new tests needed. Existing tests already verify that `user_context` is passed through verbatim (see `user_context_preserved_verbatim` test in executor.rs).

### Project Structure Notes

```
crates/bmad-types/src/
  output.rs                <- Update doc comment on AgentOutput.user_context

crates/bmad-plugin/src/
  executor.rs              <- Add doc comments on BmadOutput.user_context, .system_prompt, BmadOutputMetadata

docs/
  pulse-api-contract.md    <- Add "Input Sanitization Responsibility" section
```

### References

- `_bmad-output/planning-artifacts/epics-v2-improvements.md` lines 395-410: Story 7.4 epic definition
- `crates/bmad-plugin/src/executor.rs` lines 24-31: `BmadOutput` struct definition
- `crates/bmad-plugin/src/executor.rs` lines 116-132: `extract_user_context()` — verbatim passthrough
- `crates/bmad-types/src/output.rs` lines 26-36: `AgentOutput` struct definition with existing doc comments
- Deficiency D15 in the v2 improvements epic: "No input sanitization contract documented"

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
