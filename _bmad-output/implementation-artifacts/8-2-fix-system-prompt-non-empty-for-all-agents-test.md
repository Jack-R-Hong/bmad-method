# Story 8.2: Fix system_prompt_non_empty_for_all_agents to Actually Test All Agents

Status: done

## Story

As a QA engineer,
I want the `system_prompt_non_empty_for_all_agents` test to dynamically iterate over all registered agents,
so that newly added agents are automatically covered without manual test updates.

## Acceptance Criteria

**Given** the current test at executor.rs:272-284 hardcodes only 4 agents in a manual array
**When** the test is refactored to use `generated::all_agent_entries()`
**Then** it dynamically iterates over all agents returned by that function

**Given** the refactored test iterates over `all_agent_entries()`
**When** `cargo test -p bmad-plugin system_prompt_non_empty_for_all_agents` runs
**Then** all 12 agents are tested for non-empty system prompts without any hardcoded agent list

**Given** a new agent is added to the generated module in the future
**When** the test runs
**Then** the new agent is automatically included without any code changes to the test

**Given** any agent returns an empty or whitespace-only system prompt
**When** the test encounters that agent
**Then** the test fails with a message identifying which agent has the empty prompt

## Tasks / Subtasks

- [ ] **Task 1: Replace hardcoded agent array with `all_agent_entries()` loop** (AC: all)
  - Open `crates/bmad-plugin/src/executor.rs`, locate the `system_prompt_non_empty_for_all_agents` test (lines 272-303)
  - Remove the manual `agents` array that lists only 4 tuples
  - Replace with: `let entries = generated::all_agent_entries();`
  - Loop over `entries` using `for (meta, prompt, params) in &entries`
  - Construct `BmadExecutor::for_agent(meta, prompt, params.clone())` for each entry
  - Keep the existing assertion logic: execute, parse output, assert system_prompt is non-empty
  - Ensure the assertion message includes `meta.name` or `meta.executor_name` for diagnosis

- [ ] **Task 2: Verify agent count assertion** (AC: dynamic coverage)
  - Optionally add `assert!(entries.len() >= 12, ...)` at the top of the test to confirm the expected minimum agent count
  - This acts as a canary: if `all_agent_entries()` is accidentally empty or truncated, the test fails early

- [ ] **Task 3: Run and validate** (AC: all pass)
  - Run `cargo test -p bmad-plugin system_prompt_non_empty_for_all_agents`
  - Verify test passes for all 12 agents
  - Run `cargo test -p bmad-plugin` to confirm no regressions

## Dev Notes

The current test at executor.rs:272-284 looks like this:

```rust
fn system_prompt_non_empty_for_all_agents() {
    let agents: &[(&'static bmad_types::AgentMetadata, &'static str)] = &[
        (&generated::architect::ARCHITECT, generated::architect::SYSTEM_PROMPT),
        (&generated::developer::DEVELOPER, generated::developer::SYSTEM_PROMPT),
        (&generated::pm::PM, generated::pm::SYSTEM_PROMPT),
        (&generated::qa::QA, generated::qa::SYSTEM_PROMPT),
    ];
    // ...
}
```

The fix replaces this with `generated::all_agent_entries()` which returns `Vec<(&'static AgentMetadata, &'static str, Option<GenerationParams>)>`. Note the tuple has 3 elements (includes `Option<GenerationParams>`), so the destructuring must be updated accordingly.

The `all_agent_entries()` function is already imported via `use crate::generated;` at executor.rs:137.

This is a small, low-risk refactor. The test logic (execute + assert non-empty prompt) stays the same; only the agent iteration source changes.

### Project Structure Notes

- **Target file:** `crates/bmad-plugin/src/executor.rs` lines 272-303
- **Dynamic agent registry:** `crates/bmad-plugin/src/generated/mod.rs` -- `all_agent_entries()` at line 49
- **Return type of `all_agent_entries()`:** `Vec<(&'static AgentMetadata, &'static str, Option<GenerationParams>)>`

### References

- Epic 8 (Test Coverage Completeness) from v2 improvements epic file
- `generated::all_agent_entries()` definition at `crates/bmad-plugin/src/generated/mod.rs:49-108`

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
