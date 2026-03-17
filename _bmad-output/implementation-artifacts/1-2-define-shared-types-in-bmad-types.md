# Story 1.2: Define Shared Types in bmad-types

Status: done

## Story

As a developer,
I want all shared data structures defined once in `bmad-types`,
so that `bmad-converter` and `bmad-plugin` share identical type definitions without duplication.

## Acceptance Criteria

**AC1:**
**Given** the `bmad-types` crate exists
**When** I inspect `src/metadata.rs`
**Then** `AgentMetadata` struct is defined with fields: `id: &'static str`, `name: &'static str`, `display_name: &'static str`, `description: &'static str`, `executor_name: &'static str`, and `capabilities: &'static [&'static str]`
**And** all fields use `&'static str` for compile-time embedded data (not owned `String`)

**AC2:**
**Given** the `bmad-types` crate exists
**When** I inspect `src/output.rs`
**Then** `AgentOutput` struct contains `system_prompt: String`, `user_context: String`, and `suggested_params: Option<GenerationParams>`
**And** `GenerationParams` contains at minimum `model: Option<String>` and `temperature: Option<f32>`

**AC3:**
**Given** the `bmad-types` crate exists
**When** I inspect `src/error.rs`
**Then** `BmadError` enum is defined using `#[derive(thiserror::Error, Debug)]` with variants: `AgentNotFound(String)`, `InvalidInput(String)`, and `ExecutionFailed(String)`
**And** each variant has a human-readable `#[error(...)]` message in lowercase with no trailing punctuation

**AC4:**
**Given** `bmad-types` compiles
**When** I run `cargo test -p bmad-types`
**Then** all unit tests pass, including tests that verify `BmadError` display messages match the specified format

## Tasks / Subtasks

- [x] **Task 1: Create src/metadata.rs** (AC: #1)
  - [x] Define `AgentMetadata` struct with all 6 fields using `&'static str` / `&'static [&'static str]`
  - [x] Derive `Debug`, `Clone`, `Copy` — metadata is immutable and copyable
  - [x] Write unit test: construct `AgentMetadata` with literal values and assert all fields accessible
  - [x] Write unit test: verify `capabilities` is a slice of string references

- [x] **Task 2: Create src/output.rs** (AC: #2)
  - [x] Define `GenerationParams` struct with `model: Option<String>` and `temperature: Option<f32>`
  - [x] Derive `Debug`, `Clone`, `serde::Serialize`, `serde::Deserialize` on `GenerationParams`
  - [x] Define `AgentOutput` struct with `system_prompt: String`, `user_context: String`, `suggested_params: Option<GenerationParams>`
  - [x] Derive `Debug`, `Clone` on `AgentOutput`
  - [x] Write unit test: construct `AgentOutput` with `suggested_params: None` and `Some(...)`
  - [x] Write unit test: verify `AgentOutput` fields are owned `String`, not references

- [x] **Task 3: Create src/error.rs** (AC: #3)
  - [x] Define `BmadError` enum with `thiserror::Error` derive
  - [x] Add `AgentNotFound(String)` variant with `#[error("agent '{0}' not found")]`
  - [x] Add `InvalidInput(String)` variant with `#[error("invalid input: {0}")]`
  - [x] Add `ExecutionFailed(String)` variant with `#[error("execution failed: {0}")]`
  - [x] Derive `Debug` on the enum

- [x] **Task 4: Update src/lib.rs with re-exports** (AC: #1, #2, #3)
  - [x] Replace skeleton content with module declarations and pub use re-exports
  - [x] Re-export: `pub use metadata::AgentMetadata`
  - [x] Re-export: `pub use output::{AgentOutput, GenerationParams}`
  - [x] Re-export: `pub use error::BmadError`

- [x] **Task 5: Write error format unit tests** (AC: #4)
  - [x] Test `BmadError::AgentNotFound("architect".to_string()).to_string()` equals `"agent 'architect' not found"`
  - [x] Test `BmadError::InvalidInput("empty".to_string()).to_string()` equals `"invalid input: empty"`
  - [x] Test `BmadError::ExecutionFailed("timeout".to_string()).to_string()` equals `"execution failed: timeout"`
  - [x] Verify all messages are lowercase and have no trailing punctuation

- [x] **Task 6: Run tests and verify** (AC: #4)
  - [x] Run `cargo test -p bmad-types` — all tests must pass
  - [x] Run `cargo clippy -p bmad-types` — zero warnings
  - [x] Run `cargo doc -p bmad-types` — documentation renders without errors

## Dev Notes

### Complete src/metadata.rs

```rust
// crates/bmad-types/src/metadata.rs

/// Static metadata for a BMAD agent, embedded at compile time.
/// All fields use `&'static str` to avoid heap allocation for constant data.
#[derive(Debug, Clone, Copy)]
pub struct AgentMetadata {
    /// Internal identifier, lowercase hyphen-separated (e.g., "architect")
    pub id: &'static str,
    /// Short programmatic name (e.g., "architect")
    pub name: &'static str,
    /// Human-readable display name (e.g., "Winston the Architect")
    pub display_name: &'static str,
    /// One-line description of the agent's purpose
    pub description: &'static str,
    /// Pulse executor name in `bmad/{id}` format (e.g., "bmad/architect")
    pub executor_name: &'static str,
    /// List of capability tags for discovery
    pub capabilities: &'static [&'static str],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_metadata_construction() {
        let meta = AgentMetadata {
            id: "architect",
            name: "architect",
            display_name: "Winston the Architect",
            description: "Architecture review and design guidance",
            executor_name: "bmad/architect",
            capabilities: &["architecture", "design", "review"],
        };
        assert_eq!(meta.id, "architect");
        assert_eq!(meta.executor_name, "bmad/architect");
        assert_eq!(meta.capabilities.len(), 3);
    }

    #[test]
    fn capabilities_is_static_slice() {
        let caps: &'static [&'static str] = &["planning", "analysis"];
        let meta = AgentMetadata {
            id: "pm",
            name: "pm",
            display_name: "John the PM",
            description: "Product management tasks",
            executor_name: "bmad/pm",
            capabilities: caps,
        };
        assert!(meta.capabilities.contains(&"planning"));
    }
}
```

### Complete src/output.rs

```rust
// crates/bmad-types/src/output.rs
use serde::{Deserialize, Serialize};

/// Optional generation parameters an agent can suggest to Pulse.
/// Pulse is free to ignore these — the plugin does not own LLM execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    /// Preferred model identifier (e.g., "gpt-4", "claude-3-opus")
    pub model: Option<String>,
    /// Sampling temperature, typically 0.0–2.0
    pub temperature: Option<f32>,
    /// Maximum tokens for the response
    pub max_tokens: Option<u32>,
}

/// Structured output returned by a BMAD agent executor.
/// Pulse owns LLM execution — this struct carries prompt data only.
#[derive(Debug, Clone)]
pub struct AgentOutput {
    /// The agent's persona/role instructions for the LLM system prompt
    pub system_prompt: String,
    /// Task input as passed by the Pulse workflow, forwarded to user turn
    pub user_context: String,
    /// Optional generation parameters the agent suggests to Pulse
    pub suggested_params: Option<GenerationParams>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_output_with_no_params() {
        let output = AgentOutput {
            system_prompt: "You are a senior architect.".to_string(),
            user_context: "Review this design.".to_string(),
            suggested_params: None,
        };
        assert!(output.suggested_params.is_none());
        assert!(!output.system_prompt.is_empty());
    }

    #[test]
    fn agent_output_with_params() {
        let params = GenerationParams {
            model: Some("gpt-4".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(2048),
        };
        let output = AgentOutput {
            system_prompt: "You are a QA engineer.".to_string(),
            user_context: "Write tests for this.".to_string(),
            suggested_params: Some(params),
        };
        assert!(output.suggested_params.is_some());
        let p = output.suggested_params.unwrap();
        assert_eq!(p.model.as_deref(), Some("gpt-4"));
        assert!((p.temperature.unwrap() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn agent_output_fields_are_owned_strings() {
        // Verify fields are owned String, not references
        let s1 = String::from("prompt");
        let s2 = String::from("context");
        let output = AgentOutput {
            system_prompt: s1.clone(),
            user_context: s2.clone(),
            suggested_params: None,
        };
        // Can clone independently
        let _ = output.system_prompt.clone();
        let _ = output.user_context.clone();
    }
}
```

### Complete src/error.rs

```rust
// crates/bmad-types/src/error.rs

/// Typed errors for the bmad-method plugin.
/// Uses thiserror for stable, typed error interface at the plugin boundary.
/// Message format: lowercase, no trailing punctuation.
#[derive(thiserror::Error, Debug)]
pub enum BmadError {
    /// Returned when an executor name is not found in the registry
    #[error("agent '{0}' not found")]
    AgentNotFound(String),

    /// Returned when task input fails validation (e.g., empty input)
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Returned when agent execution encounters an internal failure
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_not_found_message() {
        let err = BmadError::AgentNotFound("architect".to_string());
        assert_eq!(err.to_string(), "agent 'architect' not found");
    }

    #[test]
    fn invalid_input_message() {
        let err = BmadError::InvalidInput("input cannot be empty".to_string());
        assert_eq!(err.to_string(), "invalid input: input cannot be empty");
    }

    #[test]
    fn execution_failed_message() {
        let err = BmadError::ExecutionFailed("timeout after 30s".to_string());
        assert_eq!(err.to_string(), "execution failed: timeout after 30s");
    }

    #[test]
    fn error_messages_are_lowercase() {
        let errors = vec![
            BmadError::AgentNotFound("test".to_string()),
            BmadError::InvalidInput("reason".to_string()),
            BmadError::ExecutionFailed("cause".to_string()),
        ];
        for err in errors {
            let msg = err.to_string();
            assert_eq!(msg, msg.to_lowercase(), "Error message must be lowercase: {}", msg);
            assert!(!msg.ends_with('.'), "Error message must not end with punctuation: {}", msg);
        }
    }
}
```

### Complete src/lib.rs (final)

```rust
// crates/bmad-types/src/lib.rs
//! Shared type definitions for the bmad-method Pulse plugin.
//! These types are used by both bmad-converter (build-time) and bmad-plugin (runtime).
//! RULE: Never duplicate these types in other crates.

pub mod error;
pub mod metadata;
pub mod output;

pub use error::BmadError;
pub use metadata::AgentMetadata;
pub use output::{AgentOutput, GenerationParams};
```

### Key Rules for This Story

**`&'static str` vs `String` — the critical distinction:**
- `AgentMetadata` fields → ALL `&'static str` — data is embedded at compile time in the binary
- `AgentOutput` fields → owned `String` — data is constructed at runtime per execution
- `GenerationParams` → owned `String`/`Option<String>` — runtime data
- **NEVER** mix these: `AgentMetadata` with `String` fields will break the static embedding approach

**Error message format (enforced by tests):**
- All lowercase: `"agent 'foo' not found"` ✓ — `"Agent 'foo' not found"` ✗
- No trailing period: `"invalid input: empty"` ✓ — `"invalid input: empty."` ✗
- Include context: `"agent '{0}' not found"` ✓ — `"agent not found"` ✗

**Module organization — must match architecture.md exactly:**
- `src/metadata.rs` → `AgentMetadata`
- `src/output.rs` → `AgentOutput`, `GenerationParams`
- `src/error.rs` → `BmadError`
- `src/lib.rs` → re-exports only, no type definitions in lib.rs itself

**Serde on GenerationParams:** Required because it may be serialized when Pulse processes the output. `AgentOutput` does NOT need Serde — only `GenerationParams` does.

**Derives required:**
- `AgentMetadata`: `Debug, Clone, Copy` (copyable because all `&'static str`)
- `AgentOutput`: `Debug, Clone` (NOT Copy — owns `String` data)
- `GenerationParams`: `Debug, Clone, Serialize, Deserialize`
- `BmadError`: `Debug` (thiserror adds `Display` automatically)

### Project Structure Notes

This story modifies only the `crates/bmad-types/` directory:

```
crates/bmad-types/
├── Cargo.toml          ← Already created in Story 1.1, add serde features
└── src/
    ├── lib.rs          ← Replace skeleton with module declarations + re-exports
    ├── metadata.rs     ← NEW: AgentMetadata struct
    ├── output.rs       ← NEW: AgentOutput + GenerationParams structs
    └── error.rs        ← NEW: BmadError enum
```

No other crates are modified in this story. `bmad-converter` and `bmad-plugin` both depend on `bmad-types` and will import these types in later stories.

### References

- [Source: architecture.md#Type-Definition-Patterns] — static str vs owned String rule
- [Source: architecture.md#Error-Handling-Patterns] — message format: lowercase, no trailing punctuation
- [Source: architecture.md#Plugin-Error-Types] — exact BmadError enum definition
- [Source: architecture.md#LLM-Integration-Architecture] — AgentOutput contract
- [Source: architecture.md#Architectural-Boundaries] — bmad-types has serde + thiserror deps
- [Source: epics.md#Story-1.2] — all acceptance criteria
- [Source: prd.md#Agent-API-Surface] — AgentOutput contract (returns prompt, not LLM response)

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- `cargo test -p bmad-types`: 9 tests passed (metadata: 2, output: 3, error: 4)
- `cargo clippy -p bmad-types`: zero warnings
- `cargo build --workspace`: all crates compiled successfully

### Completion Notes List

- All 6 tasks completed as specified in story
- `AgentMetadata` uses `&'static str` fields with `Debug, Clone, Copy` derives
- `AgentOutput` uses owned `String` fields with `Debug, Clone` (no Copy, no Serde)
- `GenerationParams` derives `Debug, Clone, Serialize, Deserialize`
- `BmadError` error messages are lowercase with no trailing punctuation (enforced by tests)
- `src/lib.rs` contains only module declarations and re-exports (no type definitions)

### File List

- `crates/bmad-types/src/metadata.rs` — Created: AgentMetadata struct
- `crates/bmad-types/src/output.rs` — Created: AgentOutput and GenerationParams structs
- `crates/bmad-types/src/error.rs` — Created: BmadError enum
- `crates/bmad-types/src/lib.rs` — Modified: replaced skeleton with module decls + re-exports

### Change Log

| Date | Change |
|------|--------|
| 2026-03-17 | Created metadata.rs with AgentMetadata (Debug, Clone, Copy, &'static str fields) |
| 2026-03-17 | Created output.rs with AgentOutput (Debug, Clone) and GenerationParams (Debug, Clone, Serialize, Deserialize) |
| 2026-03-17 | Created error.rs with BmadError (thiserror, lowercase messages, no trailing punctuation) |
| 2026-03-17 | Updated lib.rs: replaced skeleton with module declarations and pub use re-exports |
| 2026-03-17 | All 9 unit tests pass; clippy zero warnings; workspace build succeeds |
