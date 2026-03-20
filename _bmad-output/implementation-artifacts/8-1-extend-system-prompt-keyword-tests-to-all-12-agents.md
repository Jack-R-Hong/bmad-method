# Story 8.1: Extend System Prompt Keyword Tests to All 12 Agents

Status: ready-for-dev

## Story

As a QA engineer,
I want keyword-based system prompt tests for all 12 BMAD agents,
so that every agent's persona identity is validated and regressions in prompt content are caught early.

## Acceptance Criteria

**Given** the existing keyword tests cover only 4 agents (architect, dev, pm, qa)
**When** new keyword tests are added for the remaining 8 agents
**Then** each test validates that the agent's SYSTEM_PROMPT contains at least one persona-specific keyword (case-insensitive)

**Given** the analyst agent
**When** its system prompt is checked
**Then** it contains at least one of: "analyst", "market", "research", "requirements"

**Given** the bmad-master agent
**When** its system prompt is checked
**Then** it contains at least one of: "orchestrat", "workflow", "master", "knowledge"

**Given** the devops agent
**When** its system prompt is checked
**Then** it contains at least one of: "pipeline", "infrastructure", "deployment", "ci"

**Given** the quick-flow agent
**When** its system prompt is checked
**Then** it contains at least one of: "quick", "lean", "spec", "rapid"

**Given** the scrum-master agent
**When** its system prompt is checked
**Then** it contains at least one of: "sprint", "agile", "scrum", "ceremony"

**Given** the security agent
**When** its system prompt is checked
**Then** it contains at least one of: "threat", "security", "vulnerabilit", "defense"

**Given** the tech-writer agent
**When** its system prompt is checked
**Then** it contains at least one of: "documentation", "clarity", "technical writ"

**Given** the ux-designer agent
**When** its system prompt is checked
**Then** it contains at least one of: "user experience", "ux", "design", "empathy"

**Given** all 12 keyword tests exist
**When** `cargo test -p bmad-plugin` runs
**Then** all 12 keyword tests pass

## Tasks / Subtasks

- [ ] **Task 1: Add analyst keyword test** (AC: analyst)
  - Add `analyst_system_prompt_contains_persona_keywords` test to `executor.rs` `#[cfg(test)] mod tests`
  - Use `generated::analyst::ANALYST`, `generated::analyst::SYSTEM_PROMPT`, `generated::analyst::suggested_params()`
  - Assert prompt_lower contains at least one of: "analyst", "market", "research", "requirements"
  - Follow the exact pattern from the existing `architect_system_prompt_contains_persona_keywords` test at line 366

- [ ] **Task 2: Add bmad-master keyword test** (AC: bmad-master)
  - Add `bmad_master_system_prompt_contains_persona_keywords` test
  - Use `generated::bmad_master::BMAD_MASTER`, `generated::bmad_master::SYSTEM_PROMPT`, `generated::bmad_master::suggested_params()`
  - Assert prompt_lower contains at least one of: "orchestrat", "workflow", "master", "knowledge"

- [ ] **Task 3: Add devops keyword test** (AC: devops)
  - Add `devops_system_prompt_contains_persona_keywords` test
  - Use `generated::devops::DEVOPS`, `generated::devops::SYSTEM_PROMPT`, `generated::devops::suggested_params()`
  - Assert prompt_lower contains at least one of: "pipeline", "infrastructure", "deployment", "ci"

- [ ] **Task 4: Add quick-flow keyword test** (AC: quick-flow)
  - Add `quick_flow_system_prompt_contains_persona_keywords` test
  - Use `generated::quick_flow::QUICK_FLOW`, `generated::quick_flow::SYSTEM_PROMPT`, `generated::quick_flow::suggested_params()`
  - Assert prompt_lower contains at least one of: "quick", "lean", "spec", "rapid"

- [ ] **Task 5: Add scrum-master keyword test** (AC: scrum-master)
  - Add `scrum_master_system_prompt_contains_persona_keywords` test
  - Use `generated::scrum_master::SCRUM_MASTER`, `generated::scrum_master::SYSTEM_PROMPT`, `generated::scrum_master::suggested_params()`
  - Assert prompt_lower contains at least one of: "sprint", "agile", "scrum", "ceremony"

- [ ] **Task 6: Add security keyword test** (AC: security)
  - Add `security_system_prompt_contains_persona_keywords` test
  - Use `generated::security::SECURITY`, `generated::security::SYSTEM_PROMPT`, `generated::security::suggested_params()`
  - Assert prompt_lower contains at least one of: "threat", "security", "vulnerabilit", "defense"

- [ ] **Task 7: Add tech-writer keyword test** (AC: tech-writer)
  - Add `tech_writer_system_prompt_contains_persona_keywords` test
  - Use `generated::tech_writer::TECH_WRITER`, `generated::tech_writer::SYSTEM_PROMPT`, `generated::tech_writer::suggested_params()`
  - Assert prompt_lower contains at least one of: "documentation", "clarity", "technical writ"

- [ ] **Task 8: Add ux-designer keyword test** (AC: ux-designer)
  - Add `ux_designer_system_prompt_contains_persona_keywords` test
  - Use `generated::ux_designer::UX_DESIGNER`, `generated::ux_designer::SYSTEM_PROMPT`, `generated::ux_designer::suggested_params()`
  - Assert prompt_lower contains at least one of: "user experience", "ux", "design", "empathy"

- [ ] **Task 9: Run full test suite** (AC: all pass)
  - Run `cargo test -p bmad-plugin` and verify all 12 keyword tests pass
  - Confirm no regressions in existing tests

## Dev Notes

All new tests go in the `#[cfg(test)] mod tests` block inside `executor.rs`. Follow the exact pattern established by the 4 existing keyword tests (lines 366-448). Each test:

1. Constructs a `BmadExecutor` using the agent's generated constants
2. Creates a `TaskInput` with the agent's executor name
3. Calls `exec.execute(task, test_config())`
4. Parses the output and lowercases the system_prompt
5. Asserts at least one keyword matches using `||` chains
6. Provides a descriptive assertion message listing expected keywords

Use substring keywords (e.g., "orchestrat" matches "orchestrator"/"orchestration", "vulnerabilit" matches "vulnerability"/"vulnerabilities") to avoid brittle exact-match failures.

The `test_task()` helper hardcodes the agent field, so construct `TaskInput` directly with `.with_input()` as done in the existing keyword tests, passing the correct executor name in the JSON.

### Project Structure Notes

- **Target file:** `crates/bmad-plugin/src/executor.rs` (test module at line 134)
- **Generated agent modules:** `crates/bmad-plugin/src/generated/{analyst,bmad_master,devops,quick_flow,scrum_master,security,tech_writer,ux_designer}.rs`
- **Module index:** `crates/bmad-plugin/src/generated/mod.rs` (all modules already exported)
- **Existing keyword tests pattern:** `executor.rs` lines 366-448

### References

- Epic 8 (Test Coverage Completeness) from v2 improvements epic file
- Existing test pattern: `architect_system_prompt_contains_persona_keywords` at executor.rs:366-382

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
