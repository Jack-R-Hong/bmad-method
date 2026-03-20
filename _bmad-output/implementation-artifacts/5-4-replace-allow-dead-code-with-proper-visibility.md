# Story 5.4: Replace #[allow(dead_code)] with Proper Visibility or #[cfg(test)]

Status: ready-for-dev

## Story

As a plugin maintainer,
I want to replace blanket `#[allow(dead_code)]` annotations with proper `#[cfg(test)]` gating or justified visibility,
so that the compiler can catch genuinely unused code and the codebase communicates intent clearly.

## Acceptance Criteria

**Given** `registry.rs` has `#[allow(dead_code)]` on `new()`, `find_agent()`, `list_agents()`, and `count()` methods of `AgentRegistry`
**When** the annotations are replaced with proper gating
**Then** methods only used in tests are gated with `#[cfg(test)]` and methods used in production have no suppression

**Given** all `#[allow(dead_code)]` annotations are removed from `registry.rs`
**When** `cargo build --workspace` is run
**Then** zero dead code warnings are emitted

**Given** test-only methods are gated with `#[cfg(test)]`
**When** the plugin is compiled for release
**Then** test-only methods are not included in the release binary

**Given** all changes are complete
**When** `cargo test --workspace` is run
**Then** all existing tests pass with zero warnings

## Tasks / Subtasks

- [ ] **Task 1: Analyze which AgentRegistry methods are used in production vs. tests** (AC: #1)
  - [ ] `AgentRegistry::new()` (line 60) — used by `global_registry()` on line 12 via `OnceLock::get_or_init`, also used by `Default` impl (line 54). This IS used in production.
  - [ ] `AgentRegistry::find_agent(&self, ...)` (line 73) — called by module-level `find_agent()` on line 20, which may be used by other modules. Check if `registry::find_agent` is called anywhere outside tests.
  - [ ] `AgentRegistry::list_agents(&self)` (line 78) — called by module-level `list_agents()` on line 16, used in `lib.rs:23` health_check. This IS used in production.
  - [ ] `AgentRegistry::count(&self)` (line 83) — only called in test `registry_has_agents` (line 116). This is test-only.

- [ ] **Task 2: Determine production vs. test usage of module-level functions** (AC: #1)
  - [ ] `list_agents()` (line 15-17) — used in `lib.rs:23` (`registry::list_agents().is_empty()`). Production.
  - [ ] `find_agent()` (line 19-21) — search for `registry::find_agent` or `find_agent(` usage outside of registry.rs tests. If only used in tests, consider gating.
  - [ ] `verify_all_agents()` (line 23-45) — check if called outside of tests. Used by `lib.rs` or exposed as public API for plugin verification.

- [ ] **Task 3: Fix `AgentRegistry::new()` — remove #[allow(dead_code)]** (AC: #1, #2)
  - [ ] `new()` is used by `global_registry()` and `Default` impl — it IS production code
  - [ ] Simply remove `#[allow(dead_code)]` from line 59 — the compiler should see the usage through `OnceLock::get_or_init(AgentRegistry::new)`
  - [ ] If the compiler still warns (because `global_registry()` is only called by `list_agents()` / `find_agent()` which might themselves appear unused), trace the call chain to confirm production usage

- [ ] **Task 4: Fix `AgentRegistry::find_agent()` — remove or gate** (AC: #1, #2)
  - [ ] If `find_agent()` (module-level, line 19-21) is used in production code, remove `#[allow(dead_code)]` from the method (line 72)
  - [ ] If `find_agent()` is only used in tests, gate the module-level function AND the method with `#[cfg(test)]`
  - [ ] The method is `pub` and called by module-level `find_agent()`, so both must be considered together

- [ ] **Task 5: Fix `AgentRegistry::list_agents()` — remove #[allow(dead_code)]** (AC: #1, #2)
  - [ ] `list_agents()` is used in production (`lib.rs:23`), so the module-level function is live
  - [ ] The method `AgentRegistry::list_agents(&self)` is called by the module-level function — remove `#[allow(dead_code)]` from line 77

- [ ] **Task 6: Fix `AgentRegistry::count()` — gate with #[cfg(test)]** (AC: #1, #3)
  - [ ] `count()` is only used in test `registry_has_agents` (line 116)
  - [ ] Add `#[cfg(test)]` to the method at line 83
  - [ ] Remove `#[allow(dead_code)]` from line 82

- [ ] **Task 7: Search for other #[allow(dead_code)] in the workspace** (AC: #2)
  - [ ] Search all `.rs` files for `#[allow(dead_code)]` — there may be instances beyond registry.rs
  - [ ] For each found instance, apply the same analysis: is it production code (remove the annotation) or test-only code (gate with `#[cfg(test)]`)?
  - [ ] Generated code in `src/generated/` should NOT be modified by hand — if generated code has the annotation, fix the converter template instead

- [ ] **Task 8: Verify clean build** (AC: #2, #4)
  - [ ] Run `cargo build --workspace` — zero dead code warnings
  - [ ] Run `cargo test --workspace` — all tests pass
  - [ ] Run `cargo clippy --workspace` — no new lints
  - [ ] Run `cargo build --workspace --release` — verify test-only methods are excluded

## Dev Notes

### Architecture Context

The `AgentRegistry` struct in `crates/bmad-plugin/src/registry.rs` wraps a `HashMap` and sorted `Vec` of agents. It has four methods, all marked `#[allow(dead_code)]`:

```
Line 59: #[allow(dead_code)] pub fn new()
Line 72: #[allow(dead_code)] pub fn find_agent()
Line 77: #[allow(dead_code)] pub fn list_agents()
Line 82: #[allow(dead_code)] pub fn count()
```

The call chain for production usage is:
```
lib.rs health_check() -> registry::list_agents() -> global_registry().list_agents()
                                                     -> GLOBAL_REGISTRY.get_or_init(AgentRegistry::new)
                                                        -> AgentRegistry::new()
                                                           -> AgentRegistry::list_agents(&self)
```

So `new()` and `list_agents()` are definitely production. The compiler may not have flagged them because `global_registry()` is a private function only called by the module-level `list_agents()` and `find_agent()` — but the module-level functions are `pub` and called from `lib.rs`.

`count()` is only ever called in `registry_has_agents` test at line 116. It should be `#[cfg(test)]`.

### Approach: Restructure into impl blocks

The cleanest approach is to split the `impl AgentRegistry` into two blocks:

```rust
impl AgentRegistry {
    pub fn new() -> Self { ... }
    pub fn find_agent(&self, ...) -> ... { ... }
    pub fn list_agents(&self) -> ... { ... }
}

#[cfg(test)]
impl AgentRegistry {
    pub fn count(&self) -> usize { ... }
}
```

This clearly separates production and test-only methods.

### Key Files to Modify

| File | Action |
|------|--------|
| `crates/bmad-plugin/src/registry.rs` | Remove 4x `#[allow(dead_code)]`, gate `count()` with `#[cfg(test)]` |

### Important Constraints

- The `dispatch()` method at line 88 is already correctly gated with `#[cfg(test)]` — do not change it
- The `Default` impl at line 52-56 calls `Self::new()` — this keeps `new()` live
- Do not change method signatures or return types — only change attributes
- Do not change visibility (`pub`) — the methods may be used by other crates in the workspace or by tests in other modules

### Verifying Test-Only Exclusion (Task 8)

To confirm `count()` is excluded from release builds:
```bash
cargo build --workspace --release 2>&1 | grep -i "dead_code\|unused"
# Should output nothing
```

### Project Structure Notes

```
crates/bmad-plugin/src/
├── registry.rs    ← the only file to modify
├── executor.rs    ← not modified
├── lib.rs         ← not modified (confirms list_agents() is used)
└── generated/     ← not modified
```

### References

- `crates/bmad-plugin/src/registry.rs:59` — `#[allow(dead_code)]` on `new()`
- `crates/bmad-plugin/src/registry.rs:72` — `#[allow(dead_code)]` on `find_agent()`
- `crates/bmad-plugin/src/registry.rs:77` — `#[allow(dead_code)]` on `list_agents()`
- `crates/bmad-plugin/src/registry.rs:82` — `#[allow(dead_code)]` on `count()`
- `crates/bmad-plugin/src/registry.rs:87-106` — `dispatch()` already correctly `#[cfg(test)]`
- `crates/bmad-plugin/src/lib.rs:23` — production usage of `registry::list_agents()`

## Dev Agent Record

### Agent Model Used
### Debug Log References
### Completion Notes List
### File List
