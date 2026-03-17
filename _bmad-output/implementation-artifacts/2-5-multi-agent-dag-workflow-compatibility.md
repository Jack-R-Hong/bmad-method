# Story 2.5: Verify Multi-Agent DAG Workflow Compatibility

Status: done

## Story

As a Pulse workflow author,
I want to chain multiple BMAD agents in a DAG workflow where later steps receive earlier steps' output as input,
so that I can build end-to-end AI-powered development pipelines using different specialist agents in sequence.

## Acceptance Criteria

**Given** a workflow definition chains `bmad/architect` → `bmad/dev` → `bmad/qa` sequentially
**When** the workflow executes
**Then** each agent executes independently and returns a valid `AgentOutput`
**And** the `user_context` of each step can carry forward output from the previous step without structural changes to the executor interface

**Given** two BMAD agent executors run in parallel (independent DAG branches)
**When** both executions complete
**Then** no shared mutable state causes data races or incorrect output
**And** each `AgentOutput` contains only the data for its own execution context

**Given** one BMAD agent step fails (returns `Err`)
**When** Pulse propagates the error
**Then** the error includes the agent name and a human-readable failure reason
**And** the executor does not leave any leaked resources or corrupted state

**Given** an integration test simulates a three-agent sequential workflow (architect → dev → qa)
**When** the test runs
**Then** all three agents produce valid non-empty `AgentOutput` structs
**And** the test completes without panics, timeouts, or resource leaks

## Tasks / Subtasks

- [x] **Task 1: Implement sequential workflow chaining test** (AC: #1, #4)
  - [x] Create integration test in `tests/plugin_integration.rs` (or add to existing test file)
  - [x] Test name: `test_three_agent_sequential_workflow`
  - [x] Simulate: architect → dev → qa sequential execution
  - [x] For each step: assert `AgentOutput.system_prompt.is_empty() == false`
  - [x] For each step: assert `AgentOutput.user_context == input_passed_to_that_step`
  - [x] Carry architect's `user_context` + system_prompt as input to dev step (simulating Pulse DAG data passing)
  - [x] Assert no panics (the test itself proves this by completing)
  - [x] Assert test completes within 1 second (no I/O delays)

- [x] **Task 2: Implement parallel execution safety test** (AC: #2)
  - [x] Test name: `test_parallel_agents_no_shared_state`
  - [x] Spawn two concurrent invocations on different agent types (e.g., architect + qa) using `std::thread::spawn`
  - [x] Each thread calls `execute()` with a different input string
  - [x] After both join, assert each `AgentOutput.user_context` matches only its thread's input
  - [x] Assert neither output contains the other thread's input data
  - [x] This test proves `AgentOutput` structs are fully independent (no aliasing)

- [x] **Task 3: Implement error propagation test with agent name context** (AC: #3)
  - [x] Test name: `test_failed_agent_step_error_contains_agent_name`
  - [x] Call `execute()` with an executor name that does not exist: `bmad/nonexistent`
  - [x] Assert `Err(BmadError::AgentNotFound(name))` is returned
  - [x] Assert `name.contains("nonexistent")` — the error carries the unrecognized identifier
  - [x] Test name: `test_empty_input_error_is_descriptive`
  - [x] Call `execute()` with valid agent but empty input
  - [x] Assert `Err(BmadError::InvalidInput(msg))` where `msg.contains("empty")`
  - [x] Assert no resources are leaked by inspecting heap allocations (if possible) or by simply ensuring no panics occur

- [x] **Task 4: Verify `AgentOutput` does not leak across calls** (AC: #2, #3)
  - [x] Test name: `test_execute_outputs_are_independent`
  - [x] Call the same executor twice with different inputs: `"input_alpha"` and `"input_beta"`
  - [x] Assert first result's `user_context == "input_alpha"`
  - [x] Assert second result's `user_context == "input_beta"`
  - [x] Assert neither result's `system_prompt` was mutated by the other call
  - [x] This proves the executor holds no mutable call-to-call state

- [x] **Task 5: Verify DAG compatibility — `user_context` carries output forward** (AC: #1)
  - [x] In the sequential workflow test, simulate how Pulse would chain steps:
    - Step 1 (architect): input = `"Design a REST API for user management"`
    - Step 2 (dev): input = `architect_output.user_context + " | implement the above design"`
    - Step 3 (qa): input = `dev_output.user_context + " | write tests for the above implementation"`
  - [x] Assert each step's `user_context` contains the accumulated context string
  - [x] This confirms the `AgentOutput` interface is compatible with Pulse's DAG data passing model
  - [x] Add comment: `// Simulates Pulse DAG chaining: previous step output becomes next step input`

- [x] **Task 6: Verify no resource leaks on error** (AC: #3)
  - [x] Test name: `test_no_resource_leak_on_agent_not_found`
  - [x] Call `execute()` 100 times with nonexistent executor name in a loop
  - [x] Assert all 100 calls return `Err(BmadError::AgentNotFound(...))`
  - [x] Assert process memory does not grow unboundedly (informal: test completes, no OOM)
  - [x] Test name: `test_no_resource_leak_on_invalid_input`
  - [x] Call `execute()` 100 times with empty input in a loop
  - [x] Assert all 100 calls return `Err(BmadError::InvalidInput(...))`

- [x] **Task 7: Run all integration tests and confirm complete suite passes** (AC: #4)
  - [x] Run: `cargo test --workspace -- --nocapture`
  - [x] All tests must pass
  - [x] No test may take >5 seconds to complete (agent execution is pure in-memory, no I/O)
  - [x] Zero warnings in test compilation

## Dev Notes

### Sequential DAG Execution Model

Pulse uses a DAG (Directed Acyclic Graph) execution model where workflow steps with dependencies execute in order, with each step's output available to dependent steps. The BMAD plugin must be compatible with this model.

**How Pulse chains steps (conceptual):**
```
Step 1: execute(architect, input="Design a REST API")
  → AgentOutput { system_prompt: "...", user_context: "Design a REST API", ... }

Step 2: execute(dev, input=prev_output.combined_context())
  → AgentOutput { system_prompt: "...", user_context: "...", ... }

Step 3: execute(qa, input=prev_output.combined_context())
  → AgentOutput { system_prompt: "...", user_context: "...", ... }
```

**The plugin's role:** Each `execute()` call is independent. The plugin does NOT manage inter-step data. Pulse owns the DAG execution and passes data between steps. The plugin must:
1. Accept any string as input (including accumulated context from previous steps)
2. Return a self-contained `AgentOutput`
3. Never persist state between calls

**Integration test simulation:**
```rust
// tests/plugin_integration.rs

#[test]
fn test_three_agent_sequential_workflow() {
    let registry = Registry::new();
    
    // Step 1: Architect
    let arch_input = "Design a REST API for user management with CRUD operations";
    let arch_output = execute_agent(&registry, "architect", arch_input)
        .expect("architect step should succeed");
    
    assert!(!arch_output.system_prompt.is_empty(), "architect system_prompt must not be empty");
    assert_eq!(arch_output.user_context, arch_input, "user_context must match input exactly");
    
    // Step 2: Developer — receives architect output as context
    // Simulating how Pulse would pass data forward in the DAG
    let dev_input = format!("{}\n\n---\nPrevious step output:\n{}", 
        "Implement the API designed in the previous step",
        arch_output.user_context
    );
    let dev_output = execute_agent(&registry, "dev", &dev_input)
        .expect("developer step should succeed");
    
    assert!(!dev_output.system_prompt.is_empty(), "developer system_prompt must not be empty");
    assert_eq!(dev_output.user_context, dev_input, "user_context must match input exactly");
    
    // Step 3: QA — receives dev output as context
    let qa_input = format!("{}\n\n---\nPrevious step output:\n{}",
        "Write test cases for the implementation in the previous step",
        dev_output.user_context
    );
    let qa_output = execute_agent(&registry, "qa", &qa_input)
        .expect("QA step should succeed");
    
    assert!(!qa_output.system_prompt.is_empty(), "QA system_prompt must not be empty");
    assert_eq!(qa_output.user_context, qa_input, "user_context must match input exactly");
    
    // All three agents produced independent, valid outputs
    assert_ne!(arch_output.system_prompt, dev_output.system_prompt, 
        "agents should have distinct system prompts");
    assert_ne!(dev_output.system_prompt, qa_output.system_prompt,
        "agents should have distinct system prompts");
}
```

### Parallel Execution Safety

The Rust type system enforces much of this, but the test makes it explicit:

```rust
#[test]
fn test_parallel_agents_no_shared_state() {
    use std::thread;
    
    // Two threads invoke different agents simultaneously
    let handle_arch = thread::spawn(|| {
        let registry = Registry::new();  // Each thread creates its own registry
        execute_agent(&registry, "architect", "input_for_architect_thread")
            .expect("architect parallel execution should succeed")
    });
    
    let handle_qa = thread::spawn(|| {
        let registry = Registry::new();
        execute_agent(&registry, "qa", "input_for_qa_thread")
            .expect("qa parallel execution should succeed")
    });
    
    let arch_output = handle_arch.join().expect("architect thread should not panic");
    let qa_output = handle_qa.join().expect("qa thread should not panic");
    
    // Each output contains ONLY its thread's input
    assert_eq!(arch_output.user_context, "input_for_architect_thread");
    assert_eq!(qa_output.user_context, "input_for_qa_thread");
    
    // Outputs don't contain the other thread's data
    assert!(!arch_output.user_context.contains("qa_thread"), 
        "architect output must not contain qa thread data");
    assert!(!qa_output.user_context.contains("architect_thread"),
        "qa output must not contain architect thread data");
}
```

**Why this works with stateless executors:** Each `AgentOutput` is a freshly allocated struct. The `system_prompt` is `.to_string()` from a `&'static str` (safe, no aliasing). The `user_context` is `.to_string()` from the input parameter (also safe). No global state is modified.

### Error Propagation with Agent Context

The `BmadError` variants carry the failing agent name by design:

```rust
// This error:
Err(BmadError::AgentNotFound("unknown-agent".to_string()))

// Displays as (via thiserror #[error] annotation):
"agent 'unknown-agent' not found"

// This error:  
Err(BmadError::InvalidInput("input cannot be empty".to_string()))

// Displays as:
"invalid input: input cannot be empty"
```

When Pulse propagates the error up the DAG, the agent name is embedded in the error message. If Pulse also wraps errors with step context, the combined message becomes:
`"Step 'design' failed: agent 'unknown-agent' not found"`

This satisfies AC #3: "the error includes the agent name and a human-readable failure reason".

**Error propagation test:**
```rust
#[test]
fn test_failed_agent_step_error_contains_agent_name() {
    let registry = Registry::new();
    
    let result = execute_agent(&registry, "nonexistent-agent", "some input");
    
    assert!(result.is_err(), "nonexistent agent should return Err");
    
    match result {
        Err(BmadError::AgentNotFound(name)) => {
            assert!(
                name.contains("nonexistent-agent"),
                "error must contain the unrecognized agent name, got: '{}'",
                name
            );
        }
        other => panic!("expected AgentNotFound, got: {:?}", other),
    }
}
```

### No Resource Leaks

Since agent execution involves only heap string allocations (no file handles, no network connections, no database connections, no threads), "resource leak" in this context means:

1. **Memory:** `AgentOutput` is dropped when it goes out of scope — Rust's RAII handles this automatically. No manual memory management.
2. **Error paths:** `Err(BmadError::...)` returns immediately — no partially-allocated resources left behind.
3. **Panic safety:** Since we never use `unwrap()`/`expect()` in execute paths, panics should not occur. If they do, Rust's panic handler unwinds the stack and drops all values.

There is no special cleanup code needed — stateless execution with owned types means Rust's drop system handles everything.

**Verification:** The "100 calls in a loop" test is the practical verification. If memory leaked per call, running 100 iterations would eventually cause detectable slowdown or OOM. In practice, the test will complete in milliseconds with flat memory usage.

### Pulse DAG Execution Model Compatibility

From the architecture doc: *"Stateless execution — Pulse task model — No conversational state between calls"*

The BMAD plugin is fully compatible with Pulse's DAG model because:

1. **No session state:** Each `execute()` call is completely independent. Call it once or a million times — the executor state is identical.
2. **Thread-safe:** `BmadExecutor` implements `Send + Sync` (required by Pulse's parallel DAG execution). This is enforced because all fields are `Send + Sync` (the `Registry` contains `Box<dyn AgentHandler>` where handlers must be `Send + Sync`).
3. **Output carries all needed data:** `AgentOutput` is self-contained. Pulse can pass `user_context` or `system_prompt` as input to the next step without any special plugin coordination.
4. **No ordering requirements:** Steps can execute in any order without side effects. Parallel branches are safe.

**The `Send + Sync` requirement:**
```rust
// If Pulse requires TaskExecutor: Send + Sync (verify in actual pulse-api):
// This compiles only if ALL of the following are true:
// 1. BmadExecutor has no non-Send fields
// 2. BmadExecutor has no non-Sync fields
// 3. All AgentHandlers in Registry are Send + Sync

// Generated agents must be:
pub struct Architect;  // unit struct — trivially Send + Sync

// If agents had fields, they must be Send + Sync:
// pub struct Architect {
//     prompt: &'static str,  // &'static T is Send + Sync when T: Sync
// }
```

### Test File Location

Integration tests go in `tests/plugin_integration.rs` (workspace-level tests directory):

```
tests/
├── converter_integration.rs    ← Story 1.6 (existing)
└── plugin_integration.rs       ← This story's tests (create if not exists)
```

Unit tests for single-executor behavior go inline in `#[cfg(test)]` modules within `executor.rs` and `registry.rs`.

The integration test can import from the plugin:
```rust
// tests/plugin_integration.rs
use bmad_plugin::registry::Registry;
use bmad_types::{AgentOutput, BmadError};

fn execute_agent(registry: &Registry, agent_id: &str, input: &str) -> Result<AgentOutput, BmadError> {
    let agent = registry.find_agent(agent_id)
        .ok_or_else(|| BmadError::AgentNotFound(agent_id.to_string()))?;
    agent.execute_agent(input)  // or whatever the actual method is
}
```

### Pulse DAG Workflow YAML Reference

The story validates compatibility with this workflow pattern (from prd.md):

```yaml
# workflow.yaml — this is what users write; the plugin must make this work
workflow:
  name: feature-development
  steps:
    - name: design
      executor: bmad/architect
      input: "{{ context.task }}"
      
    - name: implement
      executor: bmad/dev
      depends_on: [design]
      input: "{{ steps.design.output.user_context }}"
      
    - name: test
      executor: bmad/qa
      depends_on: [implement]
      input: "{{ steps.implement.output.user_context }}"
```

The plugin's `AgentOutput` must expose `user_context` as the field Pulse uses for data passing. Verify the exact field access pattern with the real Pulse API — Pulse may access it differently (e.g., via a serialized intermediate format).

### Project Structure Notes

This story adds integration tests and verifies cross-cutting concerns across all previously implemented components:

```
tests/
└── plugin_integration.rs    ← New or updated file for DAG compatibility tests

crates/bmad-plugin/src/
├── executor.rs    ← May need Send + Sync constraint verification
└── registry.rs    ← May need Send + Sync on AgentHandler trait bound
```

**No new source files are created** in this story. It is a verification and test-writing story that confirms the complete Epic 2 implementation is correct.

**Epic 2 completion gate:** All 5 stories in Epic 2 must pass their tests for the epic to be considered complete:
- Story 2.1: Routing tests pass
- Story 2.2: I/O tests pass
- Story 2.3: Core agent tests pass
- Story 2.4: Remaining agent tests pass
- Story 2.5: DAG compatibility integration tests pass (this story)

### References

- `epics.md` lines 498–526: Story 2.5 full acceptance criteria
- `prd.md` lines 145–178: Journey 2 (Using Multiple Agents) — the exact multi-agent DAG scenario this story validates
- `prd.md` lines 316–329: Multi-agent workflow YAML example with `depends_on`
- `architecture.md` lines 56–57: "Stateless execution — Pulse task model — No conversational state between calls"
- `architecture.md` lines 61–64: Cross-cutting concerns — error handling must propagate gracefully
- `architecture.md` lines 130–133: Testing framework — integration tests in `tests/` directory
- `epics.md` line 85: "Never panic in plugin boundary code — always return `Result`"
- NFR9 (`prd.md` line 487): "Agent execution failures are gracefully handled and reported to Pulse"
- NFR10 (`prd.md` line 488): "Plugin does not crash or hang the Pulse engine under any input"
- FR13 (`prd.md` line 434): "Pulse users can chain multiple BMAD agents in a DAG workflow"

## Dev Agent Record

### Agent Model Used
claude-sonnet-4-6

### Debug Log References
None — pure test implementation, no bugs encountered.

### Completion Notes List
- Added `bmad-plugin` and `bmad-types` as dev-dependencies to root `Cargo.toml`
- Added `execute_agent` helper to `tests/plugin_integration.rs` that replicates `BmadExecutor::execute()` using only the already-public `bmad_plugin::generated` module — no production code visibility changes required
- 7 new integration tests added: sequential DAG workflow, parallel safety, error propagation (×2), output independence, resource leak (×2)
- All 87 tests pass (80 pre-existing + 7 new)

### File List
- `Cargo.toml` — added dev-dependencies
- `tests/plugin_integration.rs` — added 7 new DAG compatibility integration tests
