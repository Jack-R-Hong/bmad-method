# Story 6.1: Extend BMAD Frontmatter Schema with Config Fields

Status: ready-for-dev

## Story

As a plugin maintainer,
I want the BMAD frontmatter parser to support optional `model_tier`, `max_turns`, and `permission_mode` fields in agent `.md` files,
so that suggested configuration values are data-driven from agent definitions instead of hardcoded in Rust source.

## Acceptance Criteria

**AC1: Parser accepts new optional fields**
**Given** an agent `.md` file with `model_tier`, `max_turns`, and `permission_mode` in its YAML frontmatter
**When** the converter parses the file
**Then** `ParsedAgent` contains the parsed values in `model_tier: Option<String>`, `max_turns: Option<u32>`, `permission_mode: Option<String>`

**AC2: Fields are optional — existing agents without them still parse**
**Given** an agent `.md` file that has no `model_tier`, `max_turns`, or `permission_mode` fields
**When** the converter parses the file
**Then** `ParsedAgent.model_tier`, `ParsedAgent.max_turns`, and `ParsedAgent.permission_mode` are all `None`
**And** all existing parser tests continue to pass without modification

**AC3: All 12 agent .md files are updated with correct config values**
**Given** the following config mapping:
- `architect`: model_tier=opus, max_turns=20, permission_mode=plan
- `bmad-master`: model_tier=opus, max_turns=20, permission_mode=plan
- `developer`: model_tier=sonnet, max_turns=30, permission_mode=bypassPermissions
- `qa`: model_tier=sonnet, max_turns=15, permission_mode=plan
- All others (analyst, devops, pm, quick-flow, scrum-master, security, tech-writer, ux-designer): model_tier=sonnet, max_turns=20, permission_mode=plan
**When** the converter parses each agent file
**Then** each `ParsedAgent` has the correct values for all three config fields

**AC4: Schema documentation is updated**
**Given** the frontmatter schema docs
**When** a developer reads them
**Then** the three new optional fields are documented with types, defaults, valid values, and examples

**AC5: All converter tests pass**
**Given** the updated parser code and agent files
**When** `cargo test -p bmad-converter` is run
**Then** all tests pass, including new tests for:
- Parsing a file with all three config fields present
- Parsing a file with only some config fields present
- Parsing a file with no config fields (backward compatibility)
- Parsing each of the 12 real agent files via `parse_directory`

## Tasks / Subtasks

- [ ] **Task 1: Add fields to `FrontmatterData` serde struct** (AC: #1, #2)
  - [ ] In `crates/bmad-converter/src/parser/frontmatter.rs`, add to `FrontmatterData`:
    - `pub model_tier: Option<String>` with `#[serde(rename = "modelTier")]` (camelCase in YAML, matching the `displayName` convention)
    - `pub max_turns: Option<u32>` with `#[serde(rename = "maxTurns")]`
    - `pub permission_mode: Option<String>` with `#[serde(rename = "permissionMode")]`
  - [ ] Decision: use camelCase YAML keys (`modelTier`, `maxTurns`, `permissionMode`) to be consistent with the existing `displayName` convention, OR use snake_case (`model_tier`, `max_turns`, `permission_mode`). Whichever is chosen, document it in schema docs. Recommend snake_case since `displayName` was an inherited convention and the rest of the frontmatter uses lowercase.

- [ ] **Task 2: Add fields to `ParsedAgent` struct** (AC: #1, #2)
  - [ ] Add `pub model_tier: Option<String>`, `pub max_turns: Option<u32>`, `pub permission_mode: Option<String>` to `ParsedAgent`
  - [ ] In `parse_file()`, propagate from `fm.model_tier`, `fm.max_turns`, `fm.permission_mode` into the `ParsedAgent` return value (same pattern as `temperature: fm.temperature`)

- [ ] **Task 3: Update all 12 agent `.md` frontmatter blocks** (AC: #3)
  - [ ] `agents/architect.md` — add: `model_tier: opus`, `max_turns: 20`, `permission_mode: plan`
  - [ ] `agents/bmad-master.md` — add: `model_tier: opus`, `max_turns: 20`, `permission_mode: plan`
  - [ ] `agents/developer.md` — add: `model_tier: sonnet`, `max_turns: 30`, `permission_mode: bypassPermissions`
  - [ ] `agents/qa.md` — add: `model_tier: sonnet`, `max_turns: 15`, `permission_mode: plan`
  - [ ] `agents/analyst.md` — add: `model_tier: sonnet`, `max_turns: 20`, `permission_mode: plan`
  - [ ] `agents/devops.md` — add: `model_tier: sonnet`, `max_turns: 20`, `permission_mode: plan`
  - [ ] `agents/pm.md` — add: `model_tier: sonnet`, `max_turns: 20`, `permission_mode: plan`
  - [ ] `agents/quick-flow.md` — add: `model_tier: sonnet`, `max_turns: 20`, `permission_mode: plan`
  - [ ] `agents/scrum-master.md` — add: `model_tier: sonnet`, `max_turns: 20`, `permission_mode: plan`
  - [ ] `agents/security.md` — add: `model_tier: sonnet`, `max_turns: 20`, `permission_mode: plan`
  - [ ] `agents/tech-writer.md` — add: `model_tier: sonnet`, `max_turns: 20`, `permission_mode: plan`
  - [ ] `agents/ux-designer.md` — add: `model_tier: sonnet`, `max_turns: 20`, `permission_mode: plan`

- [ ] **Task 4: Update schema documentation** (AC: #4)
  - [ ] In `docs/bmad-frontmatter-schema.md`, add `model_tier`, `max_turns`, `permission_mode` to the "Optional Fields" table
  - [ ] Add valid values documentation: model_tier accepts "opus" | "sonnet" | "haiku"; max_turns accepts any u32; permission_mode accepts "plan" | "bypassPermissions"
  - [ ] Update the "Complete Example" to include the new fields
  - [ ] Update the "Mapping to Rust Types" tables for both `ParsedAgent` and note that `AgentMetadata` does not carry these fields (they go through code generation in Story 6.2)

- [ ] **Task 5: Add parser unit tests** (AC: #5)
  - [ ] Test: parse file with all three config fields present — assert values match
  - [ ] Test: parse file with only `model_tier` present — assert `max_turns` and `permission_mode` are `None`
  - [ ] Test: parse file with no config fields (existing test already covers this, verify it still passes)
  - [ ] Test: parse the real `agents/` directory and verify all 12 agents have non-None config fields

- [ ] **Task 6: Update `make_agent` test helpers in converter** (AC: #5)
  - [ ] Update `make_agent()` in `crates/bmad-converter/src/codegen/templates.rs` tests to include the new `ParsedAgent` fields (set to `None` for backward compat)
  - [ ] Update `make_agent()` in `crates/bmad-converter/src/codegen/writer.rs` tests similarly

## Dev Notes

### Architecture Patterns
- Error handling: `anyhow` for converter code (consistent with existing parser)
- All new fields are `Option<T>` — the parser never fails on missing optional fields
- Do NOT add these fields to `bmad_types::AgentMetadata` — config is not metadata. It flows through code generation (Story 6.2)
- The `FrontmatterData` struct uses `serde::Deserialize` with `Option` wrappers for optional fields — follow the same pattern as `temperature`

### YAML Key Naming Decision
The existing frontmatter uses `displayName` (camelCase) for historical reasons. For new fields, consider using `snake_case` (`model_tier`, `max_turns`, `permission_mode`) since the YAML keys are internal tooling, not a public API. If camelCase is chosen instead, use `#[serde(rename = "modelTier")]` etc. Either way, document the decision.

### Testing Standards
- Run `cargo test -p bmad-converter` to verify parser changes
- No `unwrap()` or `expect()` in non-test code
- All error paths return `Result::Err` with descriptive messages

### Project Structure Notes

| Path | Role |
|------|------|
| `crates/bmad-converter/src/parser/frontmatter.rs` | `FrontmatterData` and `ParsedAgent` structs, `parse_file()` |
| `crates/bmad-converter/src/codegen/templates.rs` | Code generation templates (modified in 6.2, not this story) |
| `crates/bmad-converter/src/codegen/writer.rs` | File writer (test helpers need updating) |
| `agents/*.md` | All 12 agent definition files to update |
| `docs/bmad-frontmatter-schema.md` | Schema documentation |
| `crates/bmad-types/src/output.rs` | `SuggestedConfig` struct (already exists, not modified here) |

### References
- Epic 6 definition in `_bmad-output/planning-artifacts/epics.md`
- Existing frontmatter schema: `docs/bmad-frontmatter-schema.md`
- Current hardcoded config: `crates/bmad-plugin/src/executor.rs` lines 33-62
- Dependency: None (first story in chain)
- Blocks: Story 6.2 (cannot generate config function without parsed fields)

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
