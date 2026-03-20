# Story 9.1: Add schema_version Field to BmadOutput

Status: ready-for-dev

## Story

As a Pulse platform consumer,
I want the BmadOutput JSON to include a `schema_version` field at the top level,
so that downstream systems can detect which version of the output schema they are processing and handle breaking changes gracefully.

## Acceptance Criteria

**Given** a BmadExecutor produces output for any agent
**When** the output JSON is parsed
**Then** a `schema_version` field is present at the top level with value `"1.0"`

**Given** the `schema_version` field is serialized
**When** the output JSON key order is inspected
**Then** `schema_version` appears as a top-level key (not nested inside `metadata`)

**Given** the `BmadOutput` struct has the new field
**When** `cargo build --workspace` is run
**Then** the build succeeds with zero warnings

**Given** all existing unit tests in `executor.rs` and integration tests in `plugin_integration.rs`
**When** `cargo test --workspace` is run
**Then** all tests pass, and new tests verify `schema_version` is present and equals `"1.0"`

**Given** a consumer reads `docs/pulse-api-contract.md`
**When** they look for versioning guidance
**Then** the document describes the `schema_version` field, its location, and defines what constitutes a minor version bump vs a major version bump

## Tasks / Subtasks

- [ ] **Task 1: Add `schema_version` field to `BmadOutput`** (AC: #1, #2, #3)
  - [ ] In `crates/bmad-plugin/src/executor.rs`, add `pub schema_version: String` to the `BmadOutput` struct (line 24-31)
  - [ ] Place it as the first field in the struct so serde serializes it first in JSON output (serde serializes fields in declaration order)
  - [ ] In `BmadExecutor::execute()` (line 95-106), set `schema_version: "1.0".to_string()` in the `BmadOutput` construction

- [ ] **Task 2: Add constant for schema version** (AC: #1)
  - [ ] Define `const SCHEMA_VERSION: &str = "1.0";` near the top of `executor.rs` (after imports)
  - [ ] Use this constant in the `BmadOutput` construction: `schema_version: SCHEMA_VERSION.to_string()`
  - [ ] This avoids magic string duplication and makes future version bumps a single-line change

- [ ] **Task 3: Update unit tests in `executor.rs`** (AC: #4)
  - [ ] The `parse_output()` helper (line 165-167) deserializes into `BmadOutput`, which now requires `schema_version` — all existing tests that call `parse_output()` will automatically validate the field is present (deserialization would fail otherwise)
  - [ ] Add a dedicated test `schema_version_is_present_and_correct()` that asserts `out.schema_version == "1.0"` using the `parse_output()` helper
  - [ ] Add a test `schema_version_appears_in_json_output()` that parses the raw JSON as `serde_json::Value` and asserts `value["schema_version"].as_str() == Some("1.0")` to verify it is a top-level key

- [ ] **Task 4: Update integration tests in `tests/plugin_integration.rs`** (AC: #4)
  - [ ] In `test_three_agent_sequential_workflow()`, add assertions that each agent output (`arch_out`, `dev_out`, `qa_out`) has `["schema_version"]` equal to `"1.0"`
  - [ ] Add a standalone test `test_schema_version_present_for_all_agents()` that iterates over a few agent IDs and verifies every output contains `schema_version: "1.0"`

- [ ] **Task 5: Update `docs/pulse-api-contract.md`** (AC: #5)
  - [ ] Add a new section "## Output Schema Versioning" after the "BmadExecutor Implementation Notes" section
  - [ ] Document the `schema_version` field: type (String), location (top-level in BmadOutput JSON), initial value ("1.0")
  - [ ] Define versioning policy:
    - **Minor bump** (e.g., 1.0 -> 1.1): Adding new optional or required fields, adding new enum variants. Consumers should tolerate unknown fields.
    - **Major bump** (e.g., 1.x -> 2.0): Removing fields, renaming fields, changing field types, changing the semantic meaning of existing fields.
  - [ ] Note that consumers should parse `schema_version` as a semver-like string and check the major component for compatibility

- [ ] **Task 6: Verify clean build** (AC: #3, #4)
  - [ ] Run `cargo build --workspace` — zero warnings
  - [ ] Run `cargo test --workspace` — all tests pass
  - [ ] Run `cargo clippy --workspace` — no new lints

## Dev Notes

### Architecture Context

The `BmadOutput` struct at `crates/bmad-plugin/src/executor.rs:23-31` is the API contract between the plugin and Pulse. It is serialized to JSON via `serde_json::to_string()` at line 108 and placed into `StepResult.content`. Currently, the struct has no versioning mechanism — adding, removing, or changing fields could silently break consumers.

The `schema_version` field is intentionally placed on `BmadOutput` (not inside `BmadOutputMetadata`) because it describes the shape of the entire output document, not just the metadata. It must be a top-level key so consumers can read it before attempting to parse the rest of the structure.

### Key Patterns

- The struct derives `Serialize, Deserialize` via serde — adding a field is straightforward
- Serde serializes fields in struct declaration order, so placing `schema_version` first ensures it appears first in JSON
- Use `#[serde(deny_unknown_fields)]` is NOT on `BmadOutput` (only on `BmadInput`), so adding fields is backward-compatible for deserialization
- The `parse_output()` test helper deserializes into `BmadOutput` directly, so adding a required field will cause existing tests to fail until the field is populated — this is the desired behavior (compile-time/test-time enforcement)

### Testing Standards

- Unit tests in `crates/bmad-plugin/src/executor.rs` (mod tests) use `parse_output()` to deserialize `StepResult` content into `BmadOutput`
- Integration tests in `tests/plugin_integration.rs` use `parse_output()` to deserialize into `serde_json::Value` and access fields by string key
- Both test layers must verify `schema_version` presence and value

### Project Structure Notes

| Path | Relevance |
|------|-----------|
| `crates/bmad-plugin/src/executor.rs` | `BmadOutput` struct definition (line 23-31), `BmadExecutor::execute()` construction (line 95-106), unit tests (line 134+) |
| `tests/plugin_integration.rs` | Integration tests that parse output JSON as `serde_json::Value` |
| `docs/pulse-api-contract.md` | API contract documentation to update with versioning policy |
| `crates/bmad-types/src/metadata.rs` | `AgentMetadata` — not modified in this story but referenced by Story 9.2 |

### References

- Epic 9: API Surface Evolution (v2 improvements epic)
- `docs/pulse-api-contract.md` — existing API contract documentation
- Story 9.2 depends on this story (will bump `schema_version` to "1.1")

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
