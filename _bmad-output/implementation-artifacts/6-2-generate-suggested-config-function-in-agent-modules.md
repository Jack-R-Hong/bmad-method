# Story 6.2: Generate suggested_config() Function in Agent Modules

Status: done

## Story

As a plugin maintainer,
I want the bmad-converter to generate a `suggested_config()` function in each agent's generated `.rs` file using the parsed frontmatter config fields,
so that suggested configuration is code-generated from data rather than maintained by hand.

## Acceptance Criteria

**AC1: Each generated agent module exports a `suggested_config()` function**
**Given** an agent `.md` file with `model_tier`, `max_turns`, and `permission_mode` in frontmatter
**When** the converter generates the agent's `.rs` file
**Then** the file contains a `pub fn suggested_config() -> Option<SuggestedConfig>` function
**And** it returns `Some(SuggestedConfig { model_tier: Some("..."), max_turns: Some(...), permission_mode: Some("..."), allowed_tools: None })`

**AC2: Agents without config fields generate None**
**Given** an agent `.md` file with no `model_tier`, `max_turns`, or `permission_mode` in frontmatter
**When** the converter generates the agent's `.rs` file
**Then** the `suggested_config()` function returns `None`

**AC3: Partial config fields are handled correctly**
**Given** an agent `.md` file with only some config fields (e.g., `model_tier` but not `max_turns`)
**When** the converter generates the agent's `.rs` file
**Then** the `suggested_config()` function returns `Some(SuggestedConfig { model_tier: Some("..."), max_turns: None, permission_mode: None, allowed_tools: None })`

**AC4: `all_agent_entries()` tuple includes `Option<SuggestedConfig>` as 4th element**
**Given** the generated `mod.rs`
**When** code calls `generated::all_agent_entries()`
**Then** each entry is a 4-tuple: `(&'static AgentMetadata, &'static str, Option<GenerationParams>, Option<SuggestedConfig>)`
**And** each agent's `suggested_config()` return value is correctly wired into the tuple

**AC5: Generated code imports SuggestedConfig from bmad_types**
**Given** any generated agent `.rs` file
**When** it is compiled
**Then** it imports `SuggestedConfig` from `bmad_types` alongside existing imports
**And** `mod.rs` also imports `SuggestedConfig` in its use statement

**AC6: All tests pass**
**Given** the updated converter and regenerated code
**When** `cargo test -p bmad-converter` and `cargo test -p bmad-plugin` are run
**Then** all tests pass

## Tasks / Subtasks

- [ ] **Task 1: Update `generate_agent_file()` to emit `suggested_config()` function** (AC: #1, #2, #3, #5)
  - [ ] In `crates/bmad-converter/src/codegen/templates.rs`, modify `generate_agent_file()`:
    - Update the `imports` line to: `use bmad_types::{AgentMetadata, GenerationParams, SuggestedConfig};\n\n`
    - After the existing `suggested_params_fn` block, add a `suggested_config_fn` block
    - Logic: if any of `agent.model_tier`, `agent.max_turns`, `agent.permission_mode` is `Some`, emit `Some(SuggestedConfig { ... })` with each field mapped; otherwise emit `None`
  - [ ] Pattern for generation (string formatting, not proc macros):
    ```
    pub fn suggested_config() -> Option<SuggestedConfig> {
        Some(SuggestedConfig {
            model_tier: Some("opus".to_string()),
            max_turns: Some(20),
            permission_mode: Some("plan".to_string()),
            allowed_tools: None,
        })
    }
    ```
  - [ ] For `None` case:
    ```
    pub fn suggested_config() -> Option<SuggestedConfig> {
        None
    }
    ```

- [ ] **Task 2: Update `generate_mod_file()` to include SuggestedConfig in tuple** (AC: #4, #5)
  - [ ] In `crates/bmad-converter/src/codegen/templates.rs`, modify `generate_mod_file()`:
    - Update the import line to: `use bmad_types::{AgentMetadata, GenerationParams, SuggestedConfig};\n\n`
    - Change `all_agent_entries()` return type to: `Vec<(&'static AgentMetadata, &'static str, Option<GenerationParams>, Option<SuggestedConfig>)>`
    - Add `{snake}::suggested_config()` as the 4th element in each tuple entry:
      ```
      ({snake}::metadata(), {snake}::SYSTEM_PROMPT, {snake}::suggested_params(), {snake}::suggested_config()),
      ```

- [ ] **Task 3: Regenerate all agent `.rs` files** (AC: #1, #4)
  - [ ] Run: `cargo run -p bmad-converter -- --input agents/ --output crates/bmad-plugin/src/generated/`
  - [ ] Verify all 12 agent `.rs` files now contain `pub fn suggested_config()`
  - [ ] Verify `mod.rs` has the updated `all_agent_entries()` signature with 4-tuple

- [ ] **Task 4: Update `BmadExecutor` to accept config from constructor** (AC: #4)
  - [ ] In `crates/bmad-plugin/src/executor.rs`, add `suggested_config: Option<SuggestedConfig>` field to `BmadExecutor` struct
  - [ ] Update `BmadExecutor::for_agent()` to accept a 4th parameter: `suggested_config: Option<SuggestedConfig>`
  - [ ] In `execute()`, use `self.suggested_config.clone()` instead of calling `suggested_config_for()` (line 100)
  - [ ] Note: Do NOT remove `suggested_config_for()` yet — that is Story 6.3. Just stop calling it from `execute()`.

- [ ] **Task 5: Update `lib.rs` dispatch to pass 4-tuple** (AC: #4)
  - [ ] In `crates/bmad-plugin/src/lib.rs`, update the destructuring of `all_agent_entries()`:
    - Change `let (meta, prompt, params)` to `let (meta, prompt, params, config)`
    - Change `BmadExecutor::for_agent(meta, prompt, params)` to `BmadExecutor::for_agent(meta, prompt, params, config)`

- [ ] **Task 6: Update converter codegen tests** (AC: #6)
  - [ ] Add test: `generate_agent_file` with all config fields produces `suggested_config()` returning `Some`
  - [ ] Add test: `generate_agent_file` with no config fields produces `suggested_config()` returning `None`
  - [ ] Add test: `generate_agent_file` with partial config fields produces `Some` with mixed `Some`/`None` inner fields
  - [ ] Add test: `generate_mod_file` output contains `SuggestedConfig` in the import and the tuple type
  - [ ] Update existing `make_agent()` helpers to include `model_tier: None, max_turns: None, permission_mode: None`

- [ ] **Task 7: Update plugin tests** (AC: #6)
  - [ ] Update `BmadExecutor::for_agent()` calls in `executor.rs` tests to pass the 4th parameter (use `None` for test helpers, use generated values for integration tests)
  - [ ] Verify `suggested_config` appears in output JSON for all agents
  - [ ] Verify architect output has `model_tier: "opus"` and dev output has `permission_mode: "bypassPermissions"`

## Dev Notes

### Architecture Patterns
- Code generation uses string formatting (`format!` macro), not quote/syn proc macros
- The `SuggestedConfig` struct is already defined in `crates/bmad-types/src/output.rs` with fields: `model_tier: Option<String>`, `max_turns: Option<u32>`, `permission_mode: Option<String>`, `allowed_tools: Option<Vec<String>>`
- `allowed_tools` is always `None` in generated code (no frontmatter field for it)
- Error handling: `anyhow` in converter, `thiserror` (via `WitPluginError`) in plugin
- Never panic in plugin code — all `BmadExecutor` paths return `Result`

### Generation Logic Detail
The `suggested_config_fn` generation in `generate_agent_file()` should follow this decision tree:
1. If ALL of `model_tier`, `max_turns`, `permission_mode` are `None` -> emit `None`
2. If ANY is `Some` -> emit `Some(SuggestedConfig { ... })` where each field is individually `Some(value)` or `None`

For string fields, generate `.to_string()` calls (same pattern as the existing hardcoded function in executor.rs).

### Naming Conventions
- snake_case for modules and functions: `suggested_config()`
- PascalCase for types: `SuggestedConfig`
- SCREAMING_SNAKE_CASE for constants: `ARCHITECT`, `SYSTEM_PROMPT`

### Project Structure Notes

| Path | Role | Changes |
|------|------|---------|
| `crates/bmad-converter/src/codegen/templates.rs` | Code generation templates | Add `suggested_config()` generation, update `generate_mod_file()` |
| `crates/bmad-converter/src/codegen/writer.rs` | File writer | Test helper updates only |
| `crates/bmad-plugin/src/generated/*.rs` | Generated agent modules | Regenerated (12 files + mod.rs) |
| `crates/bmad-plugin/src/executor.rs` | Runtime executor | Add `suggested_config` field, update constructor |
| `crates/bmad-plugin/src/lib.rs` | Plugin entry point | Update 3-tuple to 4-tuple destructuring |
| `crates/bmad-types/src/output.rs` | Shared types | No changes (SuggestedConfig already exists) |

### References
- Depends on: Story 6.1 (ParsedAgent must have `model_tier`, `max_turns`, `permission_mode` fields)
- Blocks: Story 6.3 (cannot remove hardcoded match until generated config is wired through)
- Current `all_agent_entries()` 3-tuple: `crates/bmad-plugin/src/generated/mod.rs` lines 49-108
- Current `suggested_config_for()` hardcoded match: `crates/bmad-plugin/src/executor.rs` lines 33-62
- Current `BmadExecutor::for_agent()` 3-param constructor: `crates/bmad-plugin/src/executor.rs` lines 71-81

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
