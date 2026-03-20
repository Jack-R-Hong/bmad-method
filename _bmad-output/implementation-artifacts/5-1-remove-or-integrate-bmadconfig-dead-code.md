# Story 5.1: Remove or Integrate BmadConfig Dead Code

Status: done

## Story

As a plugin maintainer,
I want to remove the unused BmadConfig struct and consolidate its useful logic into the active execution path,
so that the codebase has no dead code, no misleading parallel abstractions, and new contributors can follow a single clear input path.

## Acceptance Criteria

**Given** the current codebase has both `BmadConfig` (in bmad-types) and `BmadInput` (in bmad-plugin executor)
**When** a developer reads the code to understand how agent input is parsed
**Then** there is exactly one input struct (`BmadInput`) used end-to-end, with no competing abstraction

**Given** `BmadConfig` is removed from bmad-types public exports
**When** `cargo build --workspace` is run
**Then** the build succeeds with zero warnings (no dead code, no unused imports)

**Given** `BmadConfig::executor_name()` logic (prefix normalization) was useful
**When** input arrives without the `bmad/` prefix (e.g., `"agent": "architect"`)
**Then** the execution path normalizes it to `bmad/architect` before agent lookup (preserving the useful behavior from BmadConfig)

**Given** all changes are complete
**When** `cargo test --workspace` is run
**Then** all existing tests pass, and any new normalization logic has test coverage

## Tasks / Subtasks

- [x] **Task 1: Audit all usages of BmadConfig** (AC: #1, #2)
  - [x] Confirm `BmadConfig` is only used in its own tests and nowhere else in the runtime path
  - [x] Confirm `BmadInput` in `crates/bmad-plugin/src/executor.rs:8-14` is the actual struct used in `crates/bmad-plugin/src/lib.rs:51`
  - [x] Search for any `use bmad_types::BmadConfig` or `use crate::config::BmadConfig` across workspace

- [x] **Task 2: Add agent name normalization to BmadInput or the execution path** (AC: #3)
  - [x] Added `normalized_agent()` method to `BmadInput` in `crates/bmad-plugin/src/executor.rs`
  - [x] Used `normalized_agent()` in `crates/bmad-plugin/src/lib.rs` for agent lookup
  - [x] The logic matches the existing `BmadConfig::executor_name()`: if the value already starts with `"bmad/"`, keep it; otherwise prepend `"bmad/"`

- [x] **Task 3: Remove BmadConfig from bmad-types** (AC: #1, #2)
  - [x] Delete the file `crates/bmad-types/src/config.rs`
  - [x] Remove `pub mod config;` from `crates/bmad-types/src/lib.rs`
  - [x] Remove `pub use config::BmadConfig;` from `crates/bmad-types/src/lib.rs`

- [x] **Task 4: Add tests for agent name normalization** (AC: #3, #4)
  - [x] Add test: sending `"agent": "architect"` (without prefix) routes to `bmad/architect` and returns success
  - [x] Add test: sending `"agent": "bmad/architect"` (with prefix) still works
  - [x] Add test: sending `"agent": ""` returns not_found error

- [x] **Task 5: Verify clean build** (AC: #2, #4)
  - [x] `cargo build --workspace` — zero warnings
  - [x] `cargo test --workspace` — all 109 tests pass

## Dev Notes

### Architecture Context

The codebase currently has two input structs that serve overlapping purposes:

1. **`BmadConfig`** in `crates/bmad-types/src/config.rs` — defines `agent_name: String` + `context: Option<String>` + `executor_name()` helper. This struct is never used in the runtime execution path. It is only exercised by its own unit tests.

2. **`BmadInput`** in `crates/bmad-plugin/src/executor.rs:8-14` — defines `agent: String` + `prompt: Option<String>`. This is the actual struct deserialized in `crates/bmad-plugin/src/lib.rs:51` during plugin execution.

The `BmadConfig::executor_name()` method (lines 27-33 of config.rs) contains useful prefix-normalization logic that should be preserved in the active path:
```rust
pub fn executor_name(&self) -> String {
    if self.agent_name.starts_with("bmad/") {
        self.agent_name.clone()
    } else {
        format!("bmad/{}", self.agent_name)
    }
}
```

### Key Files to Modify

| File | Action |
|------|--------|
| `crates/bmad-types/src/config.rs` | DELETE entirely |
| `crates/bmad-types/src/lib.rs` | Remove `mod config` and `pub use config::BmadConfig` |
| `crates/bmad-plugin/src/executor.rs` | Add `normalized_agent()` method to `BmadInput` |
| `crates/bmad-plugin/src/lib.rs` | Use normalized agent name in lookup (line 57) |

### Important Constraints

- `BmadInput` lives in bmad-plugin, NOT bmad-types. This is correct — it is a plugin-internal concern, not a shared type.
- `BmadInput` uses `#[serde(deny_unknown_fields)]` — do not remove this.
- The agent lookup in `lib.rs:57` compares `m.executor_name == bmad_input.agent`. After normalization, this comparison must use the normalized value.
- Never panic in plugin code. The normalization logic is infallible (string operations only), so no Result needed.

### Project Structure Notes

```
crates/
├── bmad-types/src/
│   ├── lib.rs          ← remove config module + re-export
│   ├── config.rs       ← DELETE this file
│   ├── error.rs
│   ├── metadata.rs
│   ├── output.rs
│   └── verification.rs
├── bmad-plugin/src/
│   ├── lib.rs           ← normalize agent name before lookup
│   ├── executor.rs      ← add normalized_agent() to BmadInput
│   ├── registry.rs
│   └── generated/       ← do NOT modify
└── bmad-converter/      ← do NOT modify
```

### References

- `crates/bmad-types/src/config.rs` — the struct to remove (full file, 92 lines)
- `crates/bmad-types/src/lib.rs:6,12` — module declaration and re-export to remove
- `crates/bmad-plugin/src/executor.rs:8-14` — BmadInput struct (the one to keep)
- `crates/bmad-plugin/src/lib.rs:51-68` — the active execution path using BmadInput

## Dev Agent Record

### Agent Model Used
Claude Opus 4.6

### Completion Notes List
- Deleted `BmadConfig` struct and `config.rs` entirely
- Added `normalized_agent()` method to `BmadInput` preserving prefix-normalization logic
- Updated `lib.rs` execute path to use normalized agent name for lookup
- Also fixed SDK compatibility: `TaskInput.input` changed from `Option<String>` to `Option<serde_json::Value>` — updated all call sites

### File List
- `crates/bmad-types/src/config.rs` — DELETED
- `crates/bmad-types/src/lib.rs` — removed config module and re-export
- `crates/bmad-plugin/src/executor.rs` — added `normalized_agent()` to BmadInput, fixed SDK compat
- `crates/bmad-plugin/src/lib.rs` — use normalized agent name, added 3 normalization tests, fixed SDK compat
- `crates/bmad-plugin/src/registry.rs` — fixed SDK compat (with_input takes Value)
- `tests/plugin_integration.rs` — fixed SDK compat
