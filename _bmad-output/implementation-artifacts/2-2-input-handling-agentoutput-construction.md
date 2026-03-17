# Story 2.2: Implement Input Handling and AgentOutput Construction

Status: done

## Story

As a Pulse workflow author,
I want BMAD agents to correctly parse task input and return a structured `AgentOutput`,
so that agent execution produces usable prompt output that Pulse can pass to LLM execution.

## Acceptance Criteria

**Given** a workflow step passes `input: "Review this API design..."` to a BMAD agent
**When** the executor processes the task
**Then** the input text is captured in `AgentOutput.user_context`

**Given** a BMAD agent executes successfully
**When** the `AgentOutput` is returned
**Then** `system_prompt` contains the agent's persona/role instructions as a non-empty string
**And** `user_context` contains the task input exactly as passed by the workflow
**And** `suggested_params` is `Some(GenerationParams)` if the agent has preferred parameters, or `None` otherwise

**Given** a workflow step passes an empty input string
**When** the executor processes the task
**Then** it returns `Err(BmadError::InvalidInput("input cannot be empty"))` rather than returning an empty or nonsensical prompt

**Given** multiple concurrent workflow executions invoke the same agent executor
**When** all calls complete
**Then** no shared mutable state is accessed — each `AgentOutput` is constructed independently
**And** a unit test verifies this stateless behavior by constructing two `AgentOutput` values from the same executor and asserting they are independent

## Tasks / Subtasks

- [x] **Task 1: Implement `AgentOutput` construction in generated agent handlers** (AC: #1, #2)
  - [x] Each generated agent struct (e.g., `Architect`, `Developer`) implements `AgentHandler::execute(task)`
  - [x] In `execute()`, extract the user input from the task — use the exact field/method the `pulse-api` Task type exposes for input text
  - [x] Assign to `AgentOutput::user_context` verbatim — no trimming, no transformation
  - [x] Assign to `AgentOutput::system_prompt` the agent's static system prompt string (from the `.md` file body, embedded as `&'static str`)
  - [x] Assign `AgentOutput::suggested_params` based on whether the agent's frontmatter specifies preferred generation params

- [x] **Task 2: Implement empty input validation** (AC: #3)
  - [x] In every agent's `execute()` method (or in `BmadExecutor::execute()` before dispatch), check if input is empty or whitespace-only
  - [x] If empty → return `Err(BmadError::InvalidInput("input cannot be empty".to_string()))`
  - [x] If whitespace-only → treat as empty → same error
  - [x] Decide where validation lives: prefer centralizing in `BmadExecutor::execute()` before dispatch so all agents benefit

- [x] **Task 3: Verify and implement `GenerationParams` handling** (AC: #2)
  - [x] In the generated agent code, `suggested_params` is `Some(GenerationParams { model: None, temperature: Some(0.7) })` if the agent specifies a temperature in frontmatter
  - [x] If no generation params are specified in the agent's frontmatter, set `suggested_params: None`
  - [x] Do not hardcode values — read from parsed frontmatter during code generation
  - [x] `GenerationParams` struct is defined in `bmad-types/src/output.rs`: `model: Option<String>`, `temperature: Option<f32>`

- [x] **Task 4: Ensure stateless execution** (AC: #4)
  - [x] All generated agent structs must have NO mutable fields — they are pure value types
  - [x] `execute(&self, ...)` takes `&self` (immutable reference) — never `&mut self`
  - [x] `AgentOutput` is constructed fresh on every call using `AgentOutput { system_prompt: ..., user_context: ..., suggested_params: ... }`
  - [x] No `static mut`, no `Mutex`, no `RefCell`, no `Arc<Mutex<...>>` in agent handler code
  - [x] Agent structs must derive or implement `Clone` if Pulse requires it for executor registration

- [x] **Task 5: Write statelessness and I/O unit tests** (AC: #1, #2, #3, #4)
  - [x] Test: calling `execute()` with `"Review this API design..."` → `AgentOutput.user_context == "Review this API design..."`
  - [x] Test: `AgentOutput.system_prompt` is non-empty string for every registered agent
  - [x] Test: calling `execute()` with `""` → `Err(BmadError::InvalidInput("input cannot be empty"))`
  - [x] Test: calling `execute()` with `"   "` (whitespace only) → same `InvalidInput` error
  - [x] Test: constructing two `AgentOutput` from the same executor instance with different inputs → outputs are independent (different `user_context` values, same `system_prompt`)
  - [x] Test: `AgentOutput` from two separate executor calls have no aliasing — each holds its own owned `String`

- [x] **Task 6: Verify <500ms overhead constraint (NFR2)** (AC: #2)
  - [x] Confirm `execute()` does NOT perform any I/O operations (no file reads, no network calls, no database queries)
  - [x] `system_prompt` is a `&'static str` embedded at compile time — converted to `String` via `.to_string()` or `String::from()`
  - [x] `user_context` is constructed from the input string via `.to_string()` or cloning
  - [x] The only allocations are: two `String` constructions + optional `GenerationParams` struct — no loops, no parsing at runtime
  - [x] Add a comment in `execute()` body: `// No blocking I/O — all data is statically embedded (NFR2: <500ms overhead requirement)`

## Dev Notes

### AgentOutput Struct

Defined in `crates/bmad-types/src/output.rs` (from Epic 1, Story 1.2):

```rust
// crates/bmad-types/src/output.rs
pub struct AgentOutput {
    pub system_prompt: String,   // Agent persona/role instructions (non-empty)
    pub user_context: String,    // Task input exactly as passed by workflow
    pub suggested_params: Option<GenerationParams>,
}

pub struct GenerationParams {
    pub model: Option<String>,       // e.g., Some("gpt-4o") or None
    pub temperature: Option<f32>,    // e.g., Some(0.7) or None
}
```

**Key design principle:** `AgentOutput` uses **owned `String`** (not `&'static str`) because `user_context` comes from dynamic runtime input. The `system_prompt` is static content but must be returned as `String` to satisfy the struct definition.

### LLM Integration Pattern

The plugin does NOT call any LLM. It only constructs the prompt. Pulse owns LLM execution:

```
Workflow Step
    │
    ▼ execute(task_with_input)
BmadAgent::execute()
    │
    ├─ validates input not empty
    ├─ constructs AgentOutput {
    │      system_prompt: STATIC_PROMPT.to_string(),
    │      user_context: task.input().to_string(),
    │      suggested_params: Some/None
    │  }
    │
    ▼ returns Ok(AgentOutput) to Pulse
    
Pulse then calls LLM with system_prompt + user_context
```

This decoupling (from architecture.md): *"Decouples plugin from LLM provider specifics. Lets Pulse manage API keys, rate limits, model selection."*

### Generated Agent Execute Pattern

```rust
// crates/bmad-plugin/src/generated/architect.rs (example)

use bmad_types::{AgentMetadata, AgentOutput, BmadError, GenerationParams};

// Auto-generated by bmad-converter. DO NOT EDIT.
// Source: agents/architect.md
// Generated: {timestamp}

pub static METADATA: AgentMetadata = AgentMetadata {
    id: "architect",
    name: "architect",
    display_name: "Winston the Architect",
    description: "Expert software architect specializing in...",
    executor_name: "bmad/architect",
    capabilities: &["architecture-review", "system-design", "technical-decisions"],
};

const SYSTEM_PROMPT: &str = r#"
You are Winston, a meticulous software architect...
[full persona content from agents/architect.md body]
"#;

pub struct Architect;

impl Architect {
    pub fn execute_agent(&self, input: &str) -> Result<AgentOutput, BmadError> {
        // No blocking I/O — all data is statically embedded (NFR2: <500ms overhead requirement)
        Ok(AgentOutput {
            system_prompt: SYSTEM_PROMPT.to_string(),
            user_context: input.to_string(),
            suggested_params: None, // or Some(GenerationParams { ... }) if specified in frontmatter
        })
    }
}
```

### Input Extraction from Pulse Task

The exact method to extract input text from the Pulse `Task` type must be verified against the real `pulse-api` crate. Likely patterns:

```rust
// Option A: direct field access
let input = task.input.as_str();

// Option B: method call
let input = task.input();

// Option C: input is part of context map
let input = task.context.get("input").map(|v| v.as_str()).unwrap_or("");
```

Verify with actual `pulse-api` crate source. This is the same verification needed in Story 2.1.

### Empty Input Validation — Placement Decision

Validation can live in two places:

**Option A: `BmadExecutor::execute()` (recommended)** — centralized, applies to all agents:
```rust
impl TaskExecutor for BmadExecutor {
    fn execute(&self, task: &Task) -> Result<AgentOutput, BmadError> {
        let input = extract_input(task);  // verify exact API
        
        if input.trim().is_empty() {
            return Err(BmadError::InvalidInput("input cannot be empty".to_string()));
        }
        
        // ... dispatch to agent
    }
}
```

**Option B: Per-agent** — more flexible but duplicates validation logic.

Choose Option A unless the Pulse API doesn't make input available at the executor dispatch level.

### Stateless Execution — Why It Matters

NFR requirement + DAG parallel execution safety:

```rust
// CORRECT: stateless, safe for parallel execution
pub struct Architect;  // no fields = no state

impl AgentHandler for Architect {
    fn execute(&self, task: &Task) -> Result<AgentOutput, BmadError> {
        // Every call creates fresh AgentOutput — no shared state
        Ok(AgentOutput {
            system_prompt: SYSTEM_PROMPT.to_string(),
            user_context: task.input().to_string(),
            suggested_params: None,
        })
    }
}

// WRONG: mutable state breaks parallel execution
pub struct Architect {
    last_output: Option<AgentOutput>,  // ← NEVER DO THIS
}
```

The Rust borrow checker enforces correctness here: `execute(&self, ...)` with an immutable receiver prevents mutation. If the generated code ever needs `&mut self`, that is a design error.

### Performance Budget

NFR2: `<500ms overhead beyond LLM response time`

The execute path must only perform:
1. `input.trim().is_empty()` check — O(n) string scan, negligible
2. `SYSTEM_PROMPT.to_string()` — one allocation, O(n) where n = prompt length (~1-5KB)
3. `input.to_string()` — one allocation, O(n) where n = input length
4. `AgentOutput { ... }` struct construction — stack allocation

Total overhead: two heap allocations of small strings. Well within 500ms even on the slowest compatible hardware.

**Explicitly prohibited in `execute()`:**
- File I/O (`std::fs::read`, `File::open`, etc.)
- Network calls (any HTTP client, socket operations)
- Thread spawning or `tokio::spawn`
- `std::thread::sleep` or any blocking waits
- Mutex locking (no shared mutable state to lock)

### Project Structure Notes

This story implements input/output handling across all generated agent files and centralizes validation in the executor. Key files touched:

```
crates/
├── bmad-types/src/
│   └── output.rs          ← AgentOutput + GenerationParams (already exists from Story 1.2)
│
└── bmad-plugin/src/
    ├── executor.rs         ← Add input validation before dispatch (Story 2.1 file)
    └── generated/
        ├── architect.rs    ← execute_agent() implementation
        ├── developer.rs    ← execute_agent() implementation
        └── ...             ← All agents follow identical pattern
```

The code generator (`bmad-converter`) must produce the `execute_agent()` method body — this story may require updating the code generator templates from Story 1.4 to produce the correct execute body. If so, update `crates/bmad-converter/src/codegen/templates.rs`.

### References

- `epics.md` lines 397–425: Story 2.2 full acceptance criteria
- `architecture.md` lines 202–218: `AgentOutput` struct, `GenerationParams`, LLM integration rationale
- `architecture.md` lines 186–198: `BmadError` definition
- `architecture.md` lines 282–300: Code generation patterns, raw string literals, static embedding
- `prd.md` lines 57–58: LLM integration — plugin produces prompt, Pulse owns LLM execution
- `prd.md` lines 292–301: Agent API surface — input is task context + user prompt, output is structured response
- `epics.md` line 79: "LLM integration: Agent returns prompt only (`AgentOutput` struct) — Pulse owns LLM execution"
- NFR2 (`prd.md` line 468): "Individual agent execution adds <500ms overhead beyond LLM response time"

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6 (anthropic/claude-sonnet-4-6)

### Debug Log References

No blocking issues encountered. Tasks 1, 2, and 4 were already implemented correctly by Story 2-1. Task 3 required codegen pipeline changes. Tasks 5 and 6 added new tests and the NFR2 comment.

### Completion Notes List

- ✅ Task 1: `BmadExecutor::execute()` constructs `AgentOutput` with `system_prompt: self.system_prompt.to_string()` and `user_context: input.to_string()` — verified existing implementation correct.
- ✅ Task 2: Empty/whitespace input validation via `input.trim().is_empty()` already centralized in `BmadExecutor::execute()` — verified and confirmed working.
- ✅ Task 3: Implemented full `GenerationParams` pipeline: (a) parser now reads optional `temperature` from frontmatter, (b) codegen emits `pub fn suggested_params() -> Option<GenerationParams>` per agent, (c) `BmadExecutor` stores `Option<GenerationParams>` and passes it through in `execute()`, (d) `lib.rs` calls `agent::suggested_params()` at registration time. No hardcoding — values flow from frontmatter.
- ✅ Task 4: `BmadExecutor` struct has only `&'static` refs and `Option<GenerationParams>` (no mutable fields). `execute()` takes `&self`. No `Mutex`, `RefCell`, `Arc`, or `static mut`. Verified via `executor_is_send_and_sync` test.
- ✅ Task 5: Added 8 new statelessness/I/O unit tests: `user_context_preserved_verbatim`, `system_prompt_non_empty_for_all_agents`, `two_outputs_from_same_executor_are_independent`, `outputs_own_independent_strings`, `suggested_params_forwarded_from_constructor`, `suggested_params_none_when_not_specified`. Also added 2 codegen tests: `generate_agent_file_emits_suggested_params_none_when_no_temperature`, `generate_agent_file_emits_suggested_params_some_when_temperature_specified`.
- ✅ Task 6: Added `// No blocking I/O — all data is statically embedded (NFR2: <500ms overhead requirement)` comment to `execute()` body. Confirmed no I/O in execute path — only two `String` allocations + optional `GenerationParams` clone.
- All 67 tests pass (was 59, added 8 new tests).

### File List

- `crates/bmad-converter/src/parser/frontmatter.rs` — added `temperature: Option<f32>` to `FrontmatterData` and `ParsedAgent`; populate from frontmatter YAML
- `crates/bmad-converter/src/codegen/templates.rs` — updated `generate_agent_file()` to emit `pub fn suggested_params()` per agent; import `GenerationParams`; added 2 new tests; updated `make_agent()` helper
- `crates/bmad-converter/src/codegen/writer.rs` — updated `make_agent()` test helper to include `temperature: None`
- `crates/bmad-plugin/src/executor.rs` — added `suggested_params: Option<GenerationParams>` field; updated `for_agent()` to accept it; updated `execute()` to use `self.suggested_params.clone()`; added NFR2 comment; updated all test calls to 3-arg `for_agent()`; added 6 new statelessness/I/O tests
- `crates/bmad-plugin/src/lib.rs` — updated `try_register()` to call `agent::suggested_params()` and pass to `for_agent()`
- `crates/bmad-plugin/src/generated/architect.rs` — regenerated (added `suggested_params()` fn, updated import)
- `crates/bmad-plugin/src/generated/developer.rs` — regenerated (added `suggested_params()` fn, updated import)
- `crates/bmad-plugin/src/generated/pm.rs` — regenerated (added `suggested_params()` fn, updated import)
- `crates/bmad-plugin/src/generated/qa.rs` — regenerated (added `suggested_params()` fn, updated import)

## Code Review Record

### Review Date
2026-03-17

### Reviewer
claude-sonnet-4-6 (adversarial code review workflow)

### AC Validation
| AC | Status | Notes |
|----|--------|-------|
| AC1: Input → AgentOutput.user_context | IMPLEMENTED | `executor.rs:62` — `user_context: input.to_string()` |
| AC2: system_prompt, user_context verbatim, suggested_params Some/None | IMPLEMENTED | All fields correct, no trimming on user_context |
| AC3: Empty input → `Err(BmadError::InvalidInput("input cannot be empty"))` | IMPLEMENTED | `executor.rs:56-58`, message verified |
| AC4: Stateless concurrent execution | IMPLEMENTED | No mutable state, `Send+Sync` tested, independent outputs tested |

### Findings

#### 🟡 MEDIUM — Fixed

**M1 — `frontmatter.rs`: Missing test for `temperature` frontmatter parsing**
The GenerationParams pipeline starts at the parser but no test verified `temperature: 0.7` in YAML produced `ParsedAgent.temperature = Some(0.7)`.
**Fix:** Added `parse_file_with_temperature_field` and `parse_file_without_temperature_field_yields_none` tests to `frontmatter.rs`.

**M2 — `executor.rs`: Empty-input tests didn't verify exact error message**
AC3 specifies exact message `"input cannot be empty"`, but `execute_empty_string_returns_invalid_input` and `execute_whitespace_only_returns_invalid_input` used wildcard `_` match.
**Fix:** Updated both tests to `assert_eq!(msg, "input cannot be empty")` so any message drift breaks the test.

#### 🟢 LOW — Noted (not fixed)

**L1 — `templates.rs:36`: `assert!()` panic in `raw_str_literal()`**
Uses `assert!` in production code for a pathological input guard. Acceptable for a code-generator CLI that runs at build time — panic is visible and deterministic. No fix applied.

**L2 — `lib.rs`: Hard-coded agent list in `try_register()`**
Maintenance concern: adding a new agent requires updating the list manually. Not a bug; noted for future improvement.

### Test Count
- Before review: 67 passing
- After fixes: 69 passing (+2 new temperature parsing tests)
- Regressions: 0

### Specific MUST-DO Checks
| Check | Result |
|-------|--------|
| GenerationParams pipeline (frontmatter→codegen→executor) | ✅ Connected end-to-end |
| user_context verbatim (no trimming) | ✅ `input.to_string()` — trim only for isEmpty guard |
| Statelessness properly tested | ✅ `two_outputs_from_same_executor_are_independent`, `executor_is_send_and_sync` |
| unwrap()/expect() in production code | ✅ None — all in test blocks only |
| NFR2 comment in execute() | ✅ `executor.rs:55` |

### Updated File List
- `crates/bmad-converter/src/parser/frontmatter.rs` — added `parse_file_with_temperature_field` and `parse_file_without_temperature_field_yields_none` tests
- `crates/bmad-plugin/src/executor.rs` — updated `execute_empty_string_returns_invalid_input` and `execute_whitespace_only_returns_invalid_input` to assert exact error message

## Change Log

- 2026-03-17: Story 2-2 implemented. Added `temperature` frontmatter parsing, `suggested_params()` codegen, wired through `BmadExecutor`. Added 8 new statelessness/I/O tests. Added NFR2 comment to execute(). All 67 tests pass.
- 2026-03-17: Code review complete. Fixed 2 MEDIUM issues: added parser temperature test, strengthened empty-input error message assertions. 69 tests passing.
