# Story 7.3: Document `bypassPermissions` Rationale for Dev Agent

Status: ready-for-dev

## Story

As a Pulse platform team member,
I want to understand why the dev agent suggests `bypassPermissions` and what the security implications are,
so that I can make an informed decision about whether to honor this suggestion in my workflow engine.

## Acceptance Criteria

**Given** the dev agent's markdown definition file (`agents/developer.md`)
**When** `permission_mode: bypassPermissions` is present (currently hardcoded in `executor.rs`, will move to frontmatter after Epic 6)
**Then** an inline comment in the agent file or the code explains: the dev agent needs elevated permissions because it creates and modifies files without per-file confirmation for efficient story execution

**Given** `docs/pulse-api-contract.md`
**When** the `SuggestedConfig` section is reviewed
**Then** it contains a security note: "suggested_config values are advisory. The workflow YAML takes precedence. Consumers MUST review `permission_mode` values before auto-applying, especially `bypassPermissions`."

**Given** the `SuggestedConfig` struct in `crates/bmad-types/src/output.rs`
**When** a developer reads the doc comment on the `permission_mode` field
**Then** it states: "Advisory only. Values include `plan` (default, requires approval) and `bypassPermissions` (no approval needed — use with caution). The workflow engine decides whether to honor this."

**Given** the `suggested_config_for()` match block in `crates/bmad-plugin/src/executor.rs`
**When** a developer reads the `"dev" | "developer"` arm
**Then** an inline code comment explains why `bypassPermissions` is set for this agent

## Tasks / Subtasks

- [ ] **Task 1: Add inline comment in `executor.rs` dev agent match arm** (AC: #1, #4)
  - [ ] In `crates/bmad-plugin/src/executor.rs` lines 42-47, add a comment above or within the `"dev" | "developer"` match arm:
    ```rust
    // Dev agent uses bypassPermissions because it needs to create/modify source files,
    // run tests, and execute build commands without per-operation confirmation prompts.
    // This is advisory — the workflow YAML config takes precedence.
    // SECURITY: Consumers should only honor this in trusted, sandboxed environments.
    ```
  - [ ] Keep the comment concise but cover: why, advisory nature, security caveat

- [ ] **Task 2: Add doc comment on `SuggestedConfig.permission_mode` field** (AC: #3)
  - [ ] In `crates/bmad-types/src/output.rs`, add a doc comment to the `permission_mode` field:
    ```rust
    /// Advisory permission mode for downstream execution.
    /// Values: `"plan"` (default, requires approval for actions) or
    /// `"bypassPermissions"` (no approval — use with caution).
    /// The workflow engine decides whether to honor this suggestion.
    /// The workflow YAML config takes precedence over this value.
    pub permission_mode: Option<String>,
    ```

- [ ] **Task 3: Add security note to `docs/pulse-api-contract.md`** (AC: #2)
  - [ ] Add a "SuggestedConfig Security Notes" subsection to the API contract document
  - [ ] Include the following content:
    - `suggested_config` values are **advisory** — the plugin suggests, Pulse decides
    - The workflow YAML `config:` block takes precedence over `suggested_config`
    - Consumers MUST review `permission_mode` values before auto-applying
    - `bypassPermissions` means the agent can execute file operations, shell commands, and other actions without per-action approval
    - Only the `dev` agent uses `bypassPermissions`; all other agents use `plan`
    - Recommendation: only honor `bypassPermissions` in sandboxed or CI/CD environments where the blast radius is controlled

- [ ] **Task 4: Add rationale comment in dev agent markdown** (AC: #1)
  - [ ] In `agents/developer.md`, add a YAML comment in the frontmatter explaining the permission mode:
    ```yaml
    # permission_mode: bypassPermissions
    # Rationale: Dev agent creates/modifies files and runs commands as part of
    # story implementation. Per-operation prompts would make automated workflows
    # impractical. This is advisory — workflow YAML config takes precedence.
    ```
  - [ ] Note: YAML comments are stripped during parsing, so this is purely for human readers of the source file. The actual `permission_mode` field will be added to frontmatter in Epic 6 (Story 6.1).

## Dev Notes

### Current State

The `bypassPermissions` value is hardcoded in `crates/bmad-plugin/src/executor.rs` lines 42-46:

```rust
"dev" | "developer" => SuggestedConfig {
    model_tier: Some("sonnet".to_string()),
    max_turns: Some(30),
    permission_mode: Some("bypassPermissions".to_string()),
    allowed_tools: None,
},
```

No comment exists explaining why this agent is different from all others. No documentation warns consumers about the security implications.

### Scope Clarification

This story is documentation-only. It does NOT change any runtime behavior. The dev agent will continue to suggest `bypassPermissions`. The goal is to ensure every developer and consumer who encounters this value understands:
1. Why it exists (dev agent needs file/command access for story implementation)
2. That it is advisory (workflow YAML config takes precedence)
3. The security implications (no per-action approval)
4. When it is safe to honor (sandboxed/CI environments)

### `SuggestedConfig` Field Docs

The `SuggestedConfig` struct in `crates/bmad-types/src/output.rs` currently has minimal doc comments. The struct-level comment says "Advisory configuration for downstream claude-code steps." but individual fields like `permission_mode` have no doc comments. This story adds field-level documentation.

### Relationship to Epic 6

After Epic 6 (Stories 6.1-6.3), the `permission_mode` will move from the hardcoded match block to the agent's frontmatter YAML. The inline comment added in Task 1 will be replaced by comments in the generated code and the source frontmatter. The doc comments on the struct (Task 2) and the API contract docs (Task 3) will remain relevant regardless.

### No Tests Required

This story is pure documentation. No runtime behavior changes. No new tests needed. Existing tests in `executor.rs` already verify that the dev agent gets `bypassPermissions` (see `suggested_config_for_dev_is_sonnet_bypass` test at line 486-491).

### Project Structure Notes

```
crates/bmad-types/src/
  output.rs                <- Add doc comment on SuggestedConfig.permission_mode field

crates/bmad-plugin/src/
  executor.rs              <- Add inline comment on dev agent match arm (lines 42-47)

agents/
  developer.md             <- Add YAML comment documenting bypassPermissions rationale

docs/
  pulse-api-contract.md    <- Add "SuggestedConfig Security Notes" section
```

### References

- `_bmad-output/planning-artifacts/epics-v2-improvements.md` lines 370-392: Story 7.3 epic definition
- `crates/bmad-plugin/src/executor.rs` lines 42-46: dev agent `bypassPermissions` match arm
- `crates/bmad-types/src/output.rs` lines 16-24: `SuggestedConfig` struct definition
- `agents/developer.md`: dev agent frontmatter (no `permission_mode` field yet)
- Deficiency D14 in the v2 improvements epic: "bypassPermissions for dev agent undocumented"

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
