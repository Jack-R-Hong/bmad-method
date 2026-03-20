# Story 9.2: Expose Agent Capabilities in BmadOutput

Status: done

## Story

As a Pulse platform consumer,
I want agent capabilities to be included in the BmadOutput metadata,
so that downstream systems can discover what each agent is capable of directly from execution output without needing a separate registry query.

## Acceptance Criteria

**Given** a BmadExecutor produces output for any agent
**When** the output JSON is parsed
**Then** the `metadata` object contains a `capabilities` field that is a JSON array of strings

**Given** every agent has at least one capability defined in its `AgentMetadata`
**When** capabilities are populated in `BmadOutputMetadata`
**Then** the `capabilities` array is non-empty for every agent

**Given** the `AgentMetadata.capabilities` field is `&'static [&'static str]`
**When** capabilities are copied into `BmadOutputMetadata`
**Then** they are converted to `Vec<String>` (owned strings for serialization)

**Given** Story 9.1 introduced `schema_version: "1.0"`
**When** capabilities are added to the output metadata
**Then** `schema_version` is bumped to `"1.1"` (new field added = minor version bump per the versioning policy)

**Given** all changes are complete
**When** `cargo test --workspace` is run
**Then** all tests pass, including new tests that verify capabilities presence, non-emptiness, and correct `schema_version` value

## Tasks / Subtasks

- [ ] **Task 1: Add `capabilities` field to `BmadOutputMetadata`** (AC: #1, #3)
  - [ ] In `crates/bmad-plugin/src/executor.rs`, add `pub capabilities: Vec<String>` to the `BmadOutputMetadata` struct (currently at lines 17-21)
  - [ ] Place it after `plugin_version` to keep the existing field order stable

- [ ] **Task 2: Populate capabilities from `AgentMetadata` in `BmadExecutor::execute()`** (AC: #1, #2, #3)
  - [ ] In `BmadExecutor::execute()` (line 95-106), update the `BmadOutputMetadata` construction to include:
    ```rust
    capabilities: self.metadata.capabilities.iter().map(|s| s.to_string()).collect(),
    ```
  - [ ] This converts `&'static [&'static str]` to `Vec<String>` for serde serialization

- [ ] **Task 3: Bump `schema_version` to `"1.1"`** (AC: #4)
  - [ ] Update the `SCHEMA_VERSION` constant (introduced in Story 9.1) from `"1.0"` to `"1.1"` in `crates/bmad-plugin/src/executor.rs`
  - [ ] This is a single-line change because Story 9.1 centralized the version in a constant

- [ ] **Task 4: Update unit tests in `executor.rs`** (AC: #2, #4, #5)
  - [ ] Update the `schema_version_is_present_and_correct()` test (added in Story 9.1) to expect `"1.1"` instead of `"1.0"`
  - [ ] Add test `capabilities_present_in_output_metadata()`:
    - Execute with `TEST_META` (which has `capabilities: &["testing"]`)
    - Assert `out.metadata.capabilities == vec!["testing".to_string()]`
  - [ ] Add test `capabilities_non_empty_for_all_agents()`:
    - Iterate over the agent list used in `system_prompt_non_empty_for_all_agents()` (architect, developer, pm, qa)
    - Assert `!out.metadata.capabilities.is_empty()` for each
  - [ ] Add test `capabilities_match_agent_metadata()`:
    - Execute with `generated::architect::ARCHITECT`
    - Assert the output capabilities match `ARCHITECT.capabilities` (converted to `Vec<String>`)
  - [ ] Update `TEST_META` static (line 140-147) — it already has `capabilities: &["testing"]`, so no change needed there

- [ ] **Task 5: Update integration tests in `tests/plugin_integration.rs`** (AC: #2, #4, #5)
  - [ ] In `test_three_agent_sequential_workflow()`, add assertions that each agent output has a non-empty `capabilities` array:
    ```rust
    assert!(arch_out["metadata"]["capabilities"].as_array().unwrap().len() > 0);
    ```
  - [ ] Update any `schema_version` assertions (added in Story 9.1) to expect `"1.1"`
  - [ ] Add test `test_capabilities_are_string_arrays()` that verifies every element in the `capabilities` array is a JSON string (not null, not number)

- [ ] **Task 6: Update `docs/pulse-api-contract.md`** (AC: #4)
  - [ ] In the "Output Schema Versioning" section (added in Story 9.1), add a changelog entry:
    - `1.1`: Added `capabilities: Vec<String>` to `BmadOutputMetadata`. Lists agent capability tags from `AgentMetadata`.
  - [ ] Document the `capabilities` field in the output schema section: type (`string[]`), location (`metadata.capabilities`), guaranteed non-empty

- [ ] **Task 7: Verify clean build** (AC: #5)
  - [ ] Run `cargo build --workspace` — zero warnings
  - [ ] Run `cargo test --workspace` — all tests pass
  - [ ] Run `cargo clippy --workspace` — no new lints

## Dev Notes

### Architecture Context

`AgentMetadata` (defined in `crates/bmad-types/src/metadata.rs:6-19`) contains `capabilities: &'static [&'static str]` — a compile-time static slice of capability tags. Each of the 12 agents defines its own capabilities in its generated module. For example, the architect agent has capabilities like `["architecture", "design", "review"]` (see the test at `metadata.rs:33`).

Currently, `BmadOutputMetadata` (at `executor.rs:17-21`) only carries `persona`, `plugin_name`, and `plugin_version`. The rich capability data from `AgentMetadata` is lost at the output boundary. This story bridges that gap so Pulse consumers can discover agent capabilities from execution output.

### Dependency on Story 9.1

This story **depends on Story 9.1** being completed first. Story 9.1 introduces:
- The `schema_version` field on `BmadOutput`
- The `SCHEMA_VERSION` constant
- The versioning policy in `pulse-api-contract.md`
- Tests that verify `schema_version`

This story bumps `SCHEMA_VERSION` from `"1.0"` to `"1.1"` and updates the tests accordingly. If Story 9.1 is not complete, this story's tasks for bumping the version and updating version-related tests will not apply correctly.

### Key Patterns

- The `&'static [&'static str]` to `Vec<String>` conversion uses `.iter().map(|s| s.to_string()).collect()` — this is the idiomatic Rust pattern for this conversion
- `BmadOutputMetadata` derives `Serialize, Deserialize`, so adding a `Vec<String>` field works out of the box with serde_json (serializes as a JSON array of strings)
- The `parse_output()` test helper in unit tests deserializes into `BmadOutput` (which contains `BmadOutputMetadata`), so adding a required field to `BmadOutputMetadata` is enforced at deserialization time — existing tests will fail if the field is missing

### Testing Standards

- Unit tests use typed deserialization (`BmadOutput` struct) — structural correctness is enforced by the type system
- Integration tests use `serde_json::Value` — field presence must be asserted explicitly via `value["metadata"]["capabilities"]`
- Every agent must have at least 1 capability (this is an invariant of the BMAD agent definitions)
- Test both the typed access (`out.metadata.capabilities`) and the JSON structure (`value["metadata"]["capabilities"].as_array()`)

### Project Structure Notes

| Path | Relevance |
|------|-----------|
| `crates/bmad-plugin/src/executor.rs` | `BmadOutputMetadata` struct (line 17-21), `BmadExecutor::execute()` (line 95-106), `SCHEMA_VERSION` constant (added by 9.1), unit tests (line 134+) |
| `crates/bmad-types/src/metadata.rs` | `AgentMetadata` struct with `capabilities: &'static [&'static str]` (line 6-19) |
| `tests/plugin_integration.rs` | Integration tests that parse output JSON as `serde_json::Value` |
| `docs/pulse-api-contract.md` | API contract documentation — update versioning changelog and capabilities field docs |
| `crates/bmad-plugin/src/generated/` | Generated agent modules — each defines an `AgentMetadata` static with capabilities |

### References

- Epic 9: API Surface Evolution (v2 improvements epic)
- Story 9.1: Add schema_version Field to BmadOutput (dependency — must be completed first)
- `crates/bmad-types/src/metadata.rs` — `AgentMetadata.capabilities` field definition
- `docs/pulse-api-contract.md` — API contract and versioning policy

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
