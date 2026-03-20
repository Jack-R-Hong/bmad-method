---
stepsCompleted:
  - party-mode-review
  - deficiency-analysis
  - epic-creation
sourceAnalysis: party-mode-deep-dive
date: 2026-03-20
---

# BMAD-METHOD Pulse Plugin — Improvement Epics (v2)

## Overview

This document captures improvement epics and stories identified through a comprehensive multi-agent code review of the existing bmad-method plugin implementation. These epics address technical debt, test gaps, security concerns, and API surface improvements discovered post-initial-build.

**Relationship to existing epics:** These are additive improvements to the Epics 1–4 in `epics.md`. They assume the current codebase is the baseline.

## Deficiency Inventory

| # | Severity | Type | Summary |
|---|----------|------|---------|
| D1 | Medium | Dead Code | `BmadConfig` defined but never used in execution path |
| D2 | High | Design | `suggested_config` hardcoded in match block, not generated |
| D3 | High | Bug | `verify_all_agents()` sends invalid JSON, bypasses main execute path |
| D4 | Medium | Redundancy | `AgentMetadata.id` and `.name` always identical |
| D5 | Low | Code Smell | `#[allow(dead_code)]` on 4 registry methods masks unused code |
| D6 | High | Security | No input size validation on prompt text |
| D7 | Medium | UX | `prompt` field silently falls back to `task.description` |
| D8 | Medium | Test Gap | 8/12 agents lack system prompt keyword tests |
| D9 | Medium | Test Gap | `suggested_config_for` wildcard fallback untested |
| D10 | Medium | Test Gap | Integration tests skip `suggested_config` validation |
| D11 | Low | Test Gap | `system_prompt_non_empty_for_all_agents` only tests 4 agents |
| D12 | Medium | API | `BmadOutput` has no schema version for evolution |
| D13 | Medium | Feature | `capabilities` not exposed in `BmadOutput` |
| D14 | High | Security | `bypassPermissions` for dev agent undocumented |
| D15 | Low | Security | No input sanitization contract documented |

## Epic List

### Epic 5: Code Hygiene — Dead Code, Redundancy, and Correctness Fixes

Remove dead code paths, resolve structural redundancies, and fix the `verify_all_agents` health check to actually test the real execution path.

**Deficiencies addressed:** D1, D3, D4, D5

### Epic 6: Generated Config Pipeline — Integrate `suggested_config` Into Code Generation

Extend the converter pipeline to generate `SuggestedConfig` from agent frontmatter, eliminating the hand-maintained match block in `executor.rs`.

**Deficiencies addressed:** D2

### Epic 7: Input Validation & Security Hardening

Add input size limits, document the `bypassPermissions` rationale, clarify the prompt fallback contract, and establish an input sanitization responsibility boundary.

**Deficiencies addressed:** D6, D7, D14, D15

### Epic 8: Test Coverage Completeness

Extend test coverage to all 12 agents, test the `suggested_config` fallback behavior, and add `suggested_config` validation to integration tests.

**Deficiencies addressed:** D8, D9, D10, D11

### Epic 9: API Surface Evolution — Schema Versioning and Capabilities

Add `schema_version` and `capabilities` to `BmadOutput` so Pulse can handle format evolution and make smarter agent routing decisions.

**Deficiencies addressed:** D12, D13

---

## Epic 5: Code Hygiene — Dead Code, Redundancy, and Correctness Fixes

Remove dead code paths, resolve structural redundancies, and fix the `verify_all_agents` health check to actually test the real execution path.

### Story 5.1: Remove or Integrate `BmadConfig` Dead Code

As a plugin maintainer,
I want the unused `BmadConfig` struct either removed or integrated into the execution path,
So that new contributors are not confused by two overlapping input structs with different field names.

**Context:**
`BmadConfig` (`crates/bmad-types/src/config.rs`) defines `agent_name` + `context` + `executor_name()` helper. `BmadInput` (`crates/bmad-plugin/src/executor.rs:8-14`) defines `agent` + `prompt`. Only `BmadInput` is used in `lib.rs:51`. The two structs serve overlapping purposes but are not connected.

**Acceptance Criteria:**

**Given** `BmadConfig` is not referenced anywhere in the plugin execution path
**When** a developer reads `bmad-types/src/lib.rs`
**Then** `BmadConfig` is no longer exported (removed or gated behind a feature flag)

**Given** the `executor_name()` auto-prefix logic (`bmad/` prefix) from `BmadConfig` is useful
**When** the maintainer decides to keep this logic
**Then** it is moved into `BmadInput` or into the execution path in `lib.rs` — not left in an unused struct

**Given** `BmadConfig` is removed
**When** `cargo test --workspace` runs
**Then** all tests pass and no `config.rs` tests reference removed code

**Given** the change is complete
**When** `cargo build --workspace` runs
**Then** no warnings related to unused imports or dead code appear for `BmadConfig`

---

### Story 5.2: Fix `verify_all_agents` to Send Valid JSON Through Full Execute Path

As a plugin maintainer,
I want `verify_all_agents()` to exercise the same code path as production execution,
So that health checks actually validate what they claim to validate.

**Context:**
`registry.rs:29` sends `TaskInput::new("health-check", "ping").with_input("ping")` which is not valid `BmadInput` JSON. This bypasses JSON deserialization in `lib.rs:51` because `verify_all_agents` calls `BmadExecutor::execute()` directly, and `extract_user_context` silently falls back to raw text. If the fallback behavior is ever tightened, all health checks will break.

**Acceptance Criteria:**

**Given** `verify_all_agents()` runs for each registered agent
**When** it constructs the `TaskInput`
**Then** the input is valid `BmadInput` JSON: `{"agent": "<executor_name>", "prompt": "health-check ping"}`

**Given** `verify_all_agents()` calls `BmadExecutor::execute()`
**When** the execution completes
**Then** the result is verified to contain a non-empty `system_prompt` and the correct `agent` field in the output JSON

**Given** any agent fails verification
**When** the `VerificationResult` is returned
**Then** `passed` is `false` and `failure_reason` contains a specific, actionable error message

**Given** the fix is applied
**When** `cargo test -p bmad-plugin` runs
**Then** the existing `verify_all_agents_all_pass_with_valid_input` test still passes
**And** a new test confirms that sending invalid JSON to `verify_all_agents`'s internal path would be caught

---

### Story 5.3: Consolidate or Differentiate `AgentMetadata.id` and `.name`

As a plugin maintainer,
I want `AgentMetadata` to have no redundant fields,
So that each field carries distinct semantic meaning and doesn't confuse consumers.

**Context:**
All 12 generated agents have identical `id` and `name` values (e.g., both `"architect"`). The converter generates both from the same source field. Either merge them into one field or give them distinct semantics.

**Acceptance Criteria:**

**Given** `AgentMetadata` in `crates/bmad-types/src/metadata.rs`
**When** the redundancy is resolved
**Then** either:
  - (Option A) `id` is removed and all code references `name` only, OR
  - (Option B) `id` becomes the canonical internal identifier and `name` becomes a short human-friendly label with documented distinct usage

**Given** the field is removed or renamed
**When** `cargo build --workspace` runs
**Then** all crates compile without errors
**And** generated code from bmad-converter reflects the updated struct

**Given** the converter generates agent files
**When** the output is inspected
**Then** no field contains duplicated data from another field

---

### Story 5.4: Replace `#[allow(dead_code)]` with Proper Visibility or `#[cfg(test)]`

As a plugin maintainer,
I want registry methods to be properly gated rather than suppressing dead code warnings,
So that the compiler can warn about genuinely unused code in the future.

**Context:**
`registry.rs` lines 59, 73, 77, 83 all have `#[allow(dead_code)]` on public methods. `find_agent()`, `count()`, and `new()` are only used in tests. `list_agents()` is used by the global `list_agents()` function but only indirectly via `health_check()`.

**Acceptance Criteria:**

**Given** `AgentRegistry::count()` is only used in tests
**When** the code is refactored
**Then** it is gated with `#[cfg(test)]` instead of `#[allow(dead_code)]`

**Given** `AgentRegistry::find_agent()` is only used in tests and `dispatch()`
**When** the code is reviewed
**Then** if `dispatch()` is also test-only (`#[cfg(test)]`), then `find_agent()` is also gated with `#[cfg(test)]`

**Given** `AgentRegistry::new()` is needed by `global_registry()`
**When** the code is reviewed
**Then** `#[allow(dead_code)]` is removed because the method IS used in production code

**Given** the refactor is complete
**When** `cargo build --workspace` runs with no `#[allow(dead_code)]` on registry methods
**Then** no dead code warnings are emitted

---

## Epic 6: Generated Config Pipeline — Integrate `suggested_config` Into Code Generation

Extend the converter pipeline so `SuggestedConfig` (model tier, max turns, permission mode) is defined in agent frontmatter and generated into Rust code, eliminating the hand-maintained match block.

### Story 6.1: Extend BMAD Frontmatter Schema with Config Fields

As a build system developer,
I want BMAD agent markdown frontmatter to support `model_tier`, `max_turns`, and `permission_mode` fields,
So that per-agent configuration is defined at the source of truth alongside persona data.

**Context:**
Currently, the converter only parses `name`, `displayName`, `description`, `executor`, `capabilities`, and `temperature` from frontmatter. `SuggestedConfig` values (model tier, max turns, permission mode) are hardcoded in `executor.rs:33-62` in a match block that only covers 4 agents explicitly — the other 8 fall through to a default.

**Acceptance Criteria:**

**Given** a BMAD agent `.md` file
**When** the frontmatter contains:
```yaml
model_tier: opus
max_turns: 20
permission_mode: plan
```
**Then** the converter parser extracts these as `Option<String>`, `Option<u32>`, `Option<String>` respectively

**Given** a BMAD agent `.md` file without these optional fields
**When** the converter parses it
**Then** the fields default to `None` and the converter does not error

**Given** the frontmatter schema is updated
**When** `docs/bmad-frontmatter-schema.md` is reviewed
**Then** it documents the new optional fields with types, defaults, and examples

**Given** all existing agent `.md` files
**When** the correct config values are added to each agent's frontmatter
**Then** the values match what is currently hardcoded:
  - `architect`: model_tier=opus, max_turns=20, permission_mode=plan
  - `bmad-master`: model_tier=opus, max_turns=20, permission_mode=plan
  - `dev`: model_tier=sonnet, max_turns=30, permission_mode=bypassPermissions
  - `qa`: model_tier=sonnet, max_turns=15, permission_mode=plan
  - All others: model_tier=sonnet, max_turns=20, permission_mode=plan

---

### Story 6.2: Generate `suggested_config()` Function in Agent Modules

As a build system developer,
I want the converter to generate a `suggested_config()` function in each agent's generated `.rs` file,
So that per-agent configuration is compile-time embedded alongside metadata and system prompt.

**Acceptance Criteria:**

**Given** the converter processes an agent with `model_tier: opus`, `max_turns: 20`, `permission_mode: plan` in frontmatter
**When** the code generator runs
**Then** the generated `.rs` file contains:
```rust
pub fn suggested_config() -> Option<SuggestedConfig> {
    Some(SuggestedConfig {
        model_tier: Some("opus".to_string()),
        max_turns: Some(20),
        permission_mode: Some("plan".to_string()),
        allowed_tools: None,
    })
}
```

**Given** an agent without any config fields in frontmatter
**When** the code generator runs
**Then** the generated function returns `None`

**Given** `all_agent_entries()` in `generated/mod.rs`
**When** the return type is updated
**Then** it includes `Option<SuggestedConfig>` as a fourth tuple element:
```rust
Vec<(&'static AgentMetadata, &'static str, Option<GenerationParams>, Option<SuggestedConfig>)>
```

**Given** the generated code compiles
**When** `cargo test -p bmad-plugin` runs
**Then** all tests pass with the new tuple shape

---

### Story 6.3: Remove Hardcoded `suggested_config_for()` Match Block

As a plugin maintainer,
I want to remove the hand-maintained `suggested_config_for()` function in `executor.rs`,
So that config is sourced from a single location (generated code) and cannot drift from agent definitions.

**Acceptance Criteria:**

**Given** Story 6.2 is complete and each generated module exports `suggested_config()`
**When** `executor.rs` is updated
**Then** the `suggested_config_for()` function is removed entirely
**And** `BmadExecutor` receives the config from the generated entry tuple instead

**Given** `lib.rs` constructs `BmadExecutor::for_agent()`
**When** the constructor signature is updated
**Then** it accepts `Option<SuggestedConfig>` as a parameter
**And** the value comes from `generated::{agent}::suggested_config()`

**Given** the match block is removed
**When** `cargo build -p bmad-plugin` runs
**Then** no warnings are emitted and no hardcoded agent names remain in `executor.rs`

**Given** the change is complete
**When** `cargo test --workspace` runs
**Then** all existing `suggested_config_for_*` tests are migrated to test the generated values directly
**And** every agent's config matches the values defined in its markdown frontmatter

---

## Epic 7: Input Validation & Security Hardening

Add input size limits, document the `bypassPermissions` rationale, clarify the prompt fallback contract, and establish an input sanitization responsibility boundary.

### Story 7.1: Add Input Size Limit to `extract_user_context`

As a plugin security reviewer,
I want prompt input to be bounded by a maximum size,
So that oversized payloads cannot cause excessive memory allocation, especially in WASM environments.

**Context:**
`extract_user_context()` in `executor.rs:116-132` only checks for empty input. No upper bound exists. The text is serialized into `BmadOutput` JSON, so a 10MB prompt becomes a 10MB+ output.

**Acceptance Criteria:**

**Given** a `MAX_INPUT_LEN` constant is defined (recommended: 128KB / 131072 bytes)
**When** `extract_user_context()` receives input exceeding `MAX_INPUT_LEN`
**Then** it returns `Err(WitPluginError::invalid_input("input exceeds maximum length of 131072 bytes"))`

**Given** input is exactly at the limit
**When** `extract_user_context()` processes it
**Then** it succeeds without error

**Given** input is 1 byte over the limit
**When** `extract_user_context()` processes it
**Then** it returns the size limit error

**Given** the constant is defined
**When** a developer reviews `executor.rs`
**Then** `MAX_INPUT_LEN` is documented with a comment explaining the rationale (WASM memory constraints, output size control)

**Given** the size check is implemented
**When** `cargo test -p bmad-plugin` runs
**Then** new tests verify: at-limit input passes, over-limit input fails with descriptive error, error message contains the limit value

---

### Story 7.2: Make `prompt` Fallback Explicit with Logging

As a plugin consumer,
I want to know when the prompt field is missing and the system falls back to `task.description`,
So that I can debug unexpected output without reading plugin source code.

**Context:**
`extract_user_context()` in `executor.rs:119` does `bmad.prompt.unwrap_or_else(|| task.description.clone())` silently. This means sending `{"agent": "bmad/architect"}` without `prompt` produces output using the task description, with no indication to the caller.

**Acceptance Criteria:**

**Given** a `BmadInput` with `prompt: None`
**When** `extract_user_context()` falls back to `task.description`
**Then** a `tracing::warn!` log is emitted: `"prompt field missing in BmadInput, falling back to task.description"`

**Given** a `BmadInput` with `prompt: Some("...")`
**When** `extract_user_context()` processes it
**Then** no warning is logged

**Given** the output `BmadOutput` is returned after a fallback
**When** the consumer inspects the output
**Then** a new boolean field `used_description_fallback: bool` in `BmadOutput` is set to `true`
**Or** alternatively, the fallback behavior is documented in `docs/pulse-api-contract.md` with the explicit contract: "If `prompt` is omitted, `task.description` is used as user context"

**Given** `BmadInput` documentation
**When** a developer reads the struct definition
**Then** a doc comment on the `prompt` field explains: "Optional. If omitted, falls back to `task.description`."

---

### Story 7.3: Document `bypassPermissions` Rationale for Dev Agent

As a Pulse platform team member,
I want to understand why the dev agent suggests `bypassPermissions` and what the security implications are,
So that I can make an informed decision about whether to honor this suggestion in my workflow engine.

**Context:**
`executor.rs:42-46` sets `permission_mode: Some("bypassPermissions".to_string())` only for the dev agent. No other agent uses this mode. If Pulse's workflow engine auto-applies `suggested_config`, the dev agent would run without permission checks.

**Acceptance Criteria:**

**Given** the dev agent's frontmatter (after Story 6.1)
**When** `permission_mode: bypassPermissions` is set
**Then** an inline comment in the frontmatter explains: why the dev agent needs elevated permissions (e.g., "dev agent needs to create/modify files without per-file confirmation for efficient story execution")

**Given** `docs/pulse-api-contract.md`
**When** the `SuggestedConfig` section is reviewed
**Then** it contains a security note: "suggested_config values are advisory. The workflow YAML takes precedence. Consumers MUST review `permission_mode` values before auto-applying, especially `bypassPermissions`."

**Given** the `SuggestedConfig` struct in `output.rs`
**When** a developer reads the doc comment on `permission_mode`
**Then** it states: "Advisory only. Values include 'plan' (default, requires approval), 'bypassPermissions' (no approval needed — use with caution). The workflow engine decides whether to honor this."

---

### Story 7.4: Document Input Sanitization Responsibility Boundary

As a Pulse platform developer,
I want a clear contract about who is responsible for sanitizing prompt content,
So that security-sensitive consumers know they must sanitize before rendering `user_context` in a UI.

**Acceptance Criteria:**

**Given** `docs/pulse-api-contract.md`
**When** the output section is reviewed
**Then** it contains: "The plugin passes `user_context` through verbatim. It does NOT sanitize, escape, or filter input content. Consumers that render `user_context` or `system_prompt` in HTML or other injection-sensitive contexts MUST sanitize the content before rendering."

**Given** the `BmadOutput` struct definition
**When** a developer reads the doc comment on `user_context`
**Then** it states: "Raw task input. Not sanitized — consumer must sanitize before rendering in injection-sensitive contexts."

---

## Epic 8: Test Coverage Completeness

Extend test coverage to all 12 agents for system prompt validation, test the `suggested_config` fallback behavior, and add `suggested_config` validation to integration tests.

### Story 8.1: Extend System Prompt Keyword Tests to All 12 Agents

As a QA engineer,
I want every registered agent to have a test verifying its system prompt contains persona-relevant keywords,
So that generated prompts are validated against expected content — not just non-emptiness.

**Context:**
`executor.rs:366-448` only tests 4 agents (architect, developer, pm, qa). The remaining 8 agents (analyst, bmad-master, devops, quick-flow, scrum-master, security, tech-writer, ux-designer) have no prompt keyword validation.

**Acceptance Criteria:**

**Given** the test file `executor.rs`
**When** new keyword tests are added
**Then** every agent has a test with persona-specific keywords:

| Agent | Required Keywords (at least 1 must match) |
|-------|------------------------------------------|
| analyst | "analyst", "market", "research", "requirements" |
| bmad-master | "orchestrat", "workflow", "master", "knowledge" |
| devops | "pipeline", "infrastructure", "deployment", "ci" |
| quick-flow | "quick", "lean", "spec", "rapid" |
| scrum-master | "sprint", "agile", "scrum", "ceremony" |
| security | "threat", "security", "vulnerabilit", "defense" |
| tech-writer | "documentation", "clarity", "technical writ" |
| ux-designer | "user experience", "ux", "design", "empathy" |

**Given** all 12 keyword tests exist
**When** `cargo test -p bmad-plugin` runs
**Then** all 12 tests pass

**Given** a future agent persona change
**When** the system prompt no longer contains expected keywords
**Then** the corresponding test fails, alerting the developer to verify the change was intentional

---

### Story 8.2: Fix `system_prompt_non_empty_for_all_agents` to Actually Test All Agents

As a QA engineer,
I want the test named `system_prompt_non_empty_for_all_agents` to actually iterate all agents,
So that the test name matches its behavior and new agents are automatically covered.

**Context:**
`executor.rs:272-284` hardcodes 4 agents in a manual array instead of using `all_agent_entries()`.

**Acceptance Criteria:**

**Given** the test `system_prompt_non_empty_for_all_agents`
**When** it is refactored
**Then** it uses `generated::all_agent_entries()` to iterate all registered agents dynamically:
```rust
for (meta, prompt, _params) in generated::all_agent_entries() {
    assert!(!prompt.is_empty(), "system_prompt must be non-empty for agent {}", meta.name);
}
```

**Given** a new agent is added via the converter
**When** `cargo test -p bmad-plugin` runs
**Then** the test automatically covers the new agent without manual edits

---

### Story 8.3: Add Tests for `suggested_config` Wildcard Fallback

As a QA engineer,
I want the `_` (wildcard) branch of `suggested_config_for()` to be explicitly tested,
So that the default configuration for the 8 agents that fall through the match is verified.

**Context:**
Only architect, bmad-master, dev, and qa have `suggested_config_for` tests. Agents like analyst, devops, security, etc. use the `_` fallback but this behavior has zero test coverage.

**Acceptance Criteria:**

**Given** agents that match the `_` fallback (e.g., `bmad/analyst`, `bmad/devops`, `bmad/security`)
**When** `suggested_config_for("bmad/analyst")` is called
**Then** it returns `SuggestedConfig { model_tier: Some("sonnet"), max_turns: Some(20), permission_mode: Some("plan"), allowed_tools: None }`

**Given** at least 3 distinct fallback agents are tested
**When** `cargo test -p bmad-plugin` runs
**Then** all fallback tests pass

**Given** the fallback default values change
**When** the tests run
**Then** they fail, alerting the developer to update or verify the change

*Note: After Story 6.3, these tests migrate to testing generated values instead of the match block.*

---

### Story 8.4: Add `suggested_config` Validation to Integration Tests

As a QA engineer,
I want integration tests to verify the `suggested_config` field in agent output,
So that the full execution path — from JSON input to structured output including config — is validated end-to-end.

**Context:**
`tests/plugin_integration.rs` validates `system_prompt`, `user_context`, and `metadata` but completely ignores the `suggested_config` object in the output JSON.

**Acceptance Criteria:**

**Given** the three-agent sequential workflow test (`test_three_agent_sequential_workflow`)
**When** each agent's output is parsed
**Then** the test additionally asserts:
  - `result["suggested_config"]` is a non-null JSON object
  - `result["suggested_config"]["model_tier"]` is a non-empty string
  - `result["suggested_config"]["max_turns"]` is a positive integer

**Given** the architect agent output in integration tests
**When** `suggested_config` is inspected
**Then** `model_tier` is `"opus"` and `permission_mode` is `"plan"`

**Given** the dev agent output in integration tests
**When** `suggested_config` is inspected
**Then** `model_tier` is `"sonnet"` and `permission_mode` is `"bypassPermissions"`

**Given** the parallel agents test (`test_parallel_agents_no_shared_state`)
**When** outputs are compared
**Then** each agent's `suggested_config` matches its expected values independently

---

## Epic 9: API Surface Evolution — Schema Versioning and Capabilities

Add `schema_version` and `capabilities` to `BmadOutput` so Pulse can handle format evolution and make agent routing decisions based on capabilities.

### Story 9.1: Add `schema_version` Field to `BmadOutput`

As a Pulse workflow engine developer,
I want `BmadOutput` to include a `schema_version` field,
So that I can detect output format changes and handle backwards compatibility gracefully.

**Context:**
`BmadOutput` is the API contract between the plugin and Pulse. Currently there is no way for Pulse to know which version of the output schema it is receiving. Adding a field, removing a field, or changing a field type could silently break consumers.

**Acceptance Criteria:**

**Given** `BmadOutput` in `executor.rs`
**When** `schema_version` is added
**Then** it is a `String` field with the value `"1.0"` for the current schema

**Given** the output JSON is serialized
**When** a consumer parses it
**Then** `schema_version` is present at the top level: `{"schema_version": "1.0", "agent": "...", ...}`

**Given** the schema is versioned
**When** `docs/pulse-api-contract.md` is updated
**Then** it documents: the current schema version, what changes constitute a minor vs. major version bump, and that consumers should check `schema_version` before parsing

**Given** `schema_version` is added
**When** all existing tests run
**Then** tests that parse `BmadOutput` are updated to verify `schema_version` is present and equals `"1.0"`

---

### Story 9.2: Expose Agent `capabilities` in `BmadOutput`

As a Pulse workflow engine developer,
I want `BmadOutput` to include the agent's capability tags,
So that workflow routing logic can select agents based on required capabilities (e.g., "find an agent that can do threat-modeling").

**Context:**
`AgentMetadata` contains rich capability data:
```rust
capabilities: &["architecture-review", "system-design", "technical-decisions", ...]
```
But `BmadOutput.metadata` only carries `persona`, `plugin_name`, and `plugin_version`. Pulse cannot discover agent capabilities from the execution output.

**Acceptance Criteria:**

**Given** `BmadOutputMetadata` in `executor.rs`
**When** `capabilities` is added
**Then** it is a `Vec<String>` containing the agent's capability tags from `AgentMetadata`

**Given** the architect agent executes
**When** the output JSON is parsed
**Then** `metadata.capabilities` contains `["architecture-review", "system-design", "technical-decisions", ...]`

**Given** the capabilities are populated from `AgentMetadata`
**When** `BmadExecutor::execute()` constructs the output
**Then** capabilities are copied from `self.metadata.capabilities` (converting `&[&str]` to `Vec<String>`)

**Given** all agents execute
**When** outputs are inspected
**Then** every agent has at least 1 capability in the output metadata
**And** no agent has an empty capabilities list

**Given** the output schema changes
**When** `schema_version` is evaluated
**Then** it is bumped to `"1.1"` to reflect the addition of `capabilities` (non-breaking additive change)

---

## Priority and Sequencing

### Recommended execution order:

**Phase 1 — Fix bugs and security (Sprint 1)**
1. Story 7.1 (input size limit) — High, security
2. Story 5.2 (fix verify_all_agents) — High, correctness
3. Story 7.3 (document bypassPermissions) — High, security documentation

**Phase 2 — Improve test coverage (Sprint 1-2)**
4. Story 8.2 (fix all-agents test) — Quick win
5. Story 8.1 (keyword tests for all 12 agents) — Medium effort
6. Story 8.3 (suggested_config fallback tests) — Medium effort
7. Story 8.4 (integration test suggested_config) — Medium effort

**Phase 3 — Code hygiene (Sprint 2)**
8. Story 5.1 (remove BmadConfig dead code) — Low risk
9. Story 5.3 (id/name redundancy) — Requires converter change
10. Story 5.4 (remove allow dead_code) — Quick win

**Phase 4 — Pipeline improvement (Sprint 2-3)**
11. Story 6.1 (extend frontmatter schema) — Foundation
12. Story 6.2 (generate suggested_config) — Depends on 6.1
13. Story 6.3 (remove match block) — Depends on 6.2

**Phase 5 — API evolution (Sprint 3)**
14. Story 9.1 (schema version) — Foundation
15. Story 9.2 (capabilities in output) — Depends on 9.1
16. Story 7.2 (prompt fallback logging) — Nice to have
17. Story 7.4 (sanitization doc) — Documentation only
