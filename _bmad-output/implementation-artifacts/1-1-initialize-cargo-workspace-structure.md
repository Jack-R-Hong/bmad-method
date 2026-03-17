# Story 1.1: Initialize Cargo Workspace Structure

Status: done

## Story

As a developer,
I want a properly configured Cargo workspace with three crates (`bmad-types`, `bmad-converter`, `bmad-plugin`),
so that I can build and test each component independently with shared dependency management.

## Acceptance Criteria

**AC1:**
**Given** a fresh repository
**When** I run `cargo build --workspace`
**Then** all three crates compile without errors
**And** the workspace root `Cargo.toml` defines `[workspace]` with all three members
**And** `rust-toolchain.toml` pins a stable Rust version
**And** `.gitignore` excludes `target/`, `dist/`, and `crates/bmad-plugin/src/generated/`
**And** each crate has a skeleton `lib.rs` (or `main.rs` for converter) with a single passing unit test

**AC2:**
**Given** the workspace exists
**When** I inspect the `bmad-plugin` crate's `Cargo.toml`
**Then** `crate-type = ["cdylib"]` is set for native plugin output
**And** the crate depends on `bmad-types` via workspace path dependency

**AC3:**
**Given** any dependency is declared
**When** I inspect the workspace `Cargo.toml`
**Then** shared dependency versions are defined under `[workspace.dependencies]` and inherited by member crates

## Tasks / Subtasks

- [x] **Task 1: Create workspace root Cargo.toml** (AC: #1, #3)
  - [x] Define `[workspace]` section with `members = ["crates/bmad-types", "crates/bmad-converter", "crates/bmad-plugin"]`
  - [x] Add `resolver = "2"` for edition 2021 compatibility
  - [x] Define `[workspace.dependencies]` with all shared deps (serde, thiserror, anyhow, etc.) pinned to specific versions
  - [x] Set `[workspace.package]` with `edition = "2021"`, `version = "0.1.0"`, `rust-version = "1.75"`

- [x] **Task 2: Create rust-toolchain.toml** (AC: #1)
  - [x] Create file at repo root with `[toolchain]` section
  - [x] Set `channel = "stable"` with a specific version (e.g., `"1.82.0"`)
  - [x] Add `components = ["rustfmt", "clippy"]`

- [x] **Task 3: Create directory structure** (AC: #1)
  - [x] Create `crates/bmad-types/src/`
  - [x] Create `crates/bmad-converter/src/`
  - [x] Create `crates/bmad-plugin/src/`
  - [x] Create `agents/` directory
  - [x] Create `scripts/` directory
  - [x] Create `tests/` directory
  - [x] Create `dist/` directory placeholder (add `.gitkeep`)

- [x] **Task 4: Create bmad-types crate** (AC: #1)
  - [x] Write `crates/bmad-types/Cargo.toml` with `[package]` inheriting from workspace
  - [x] Declare dependencies: `serde = { workspace = true }`, `thiserror = { workspace = true }`
  - [x] Write `crates/bmad-types/src/lib.rs` with a skeleton `// TODO: types` comment and `#[cfg(test)] mod tests { #[test] fn it_works() { assert_eq!(2 + 2, 4); } }`

- [x] **Task 5: Create bmad-converter crate** (AC: #1)
  - [x] Write `crates/bmad-converter/Cargo.toml` — `[[bin]]` section with `name = "bmad-converter"`, `path = "src/main.rs"`
  - [x] Also include `[lib]` section for testable library API: `name = "bmad_converter_lib"`, `path = "src/lib.rs"`
  - [x] Declare dependencies: `anyhow = { workspace = true }`, `bmad-types = { path = "../bmad-types" }`
  - [x] Write `crates/bmad-converter/src/main.rs` with `fn main() { println!("bmad-converter"); }` and a test
  - [x] Write `crates/bmad-converter/src/lib.rs` with `pub fn placeholder() {}` and a test

- [x] **Task 6: Create bmad-plugin crate** (AC: #1, #2)
  - [x] Write `crates/bmad-plugin/Cargo.toml` with `[lib]` section: `name = "bmad_plugin"`, `crate-type = ["cdylib"]`
  - [x] Declare workspace path dependency: `bmad-types = { path = "../bmad-types" }`
  - [x] Write `crates/bmad-plugin/src/lib.rs` with a single public function stub and a passing test

- [x] **Task 7: Create .gitignore** (AC: #1)
  - [x] Add `target/` entry
  - [x] Add `dist/` entry
  - [x] Add `crates/bmad-plugin/src/generated/` entry
  - [x] Add standard Rust/editor ignores (`.DS_Store`, `*.swp`, `Cargo.lock` optional)

- [x] **Task 8: Verify build** (AC: #1)
  - [x] Run `cargo build --workspace` — confirm zero errors
  - [x] Run `cargo test --workspace` — confirm all skeleton tests pass

## Dev Notes

### Workspace Root Cargo.toml — Exact Structure

```toml
[workspace]
members = [
    "crates/bmad-types",
    "crates/bmad-converter",
    "crates/bmad-plugin",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["BMAD-METHOD Contributors"]
license = "MIT"

[workspace.dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Frontmatter parsing
yaml-front-matter = "0.1"

# Code generation (converter only)
quote = "1.0"
proc-macro2 = "1.0"

# CLI (converter only)
clap = { version = "4.0", features = ["derive"] }

# Plugin API (plugin only — provided by Pulse)
# pulse-api = { version = "0.1" }  # Uncomment when crate is available

# Dynamic loading (integration tests)
libloading = "0.8"
```

### rust-toolchain.toml Content

```toml
[toolchain]
channel = "1.82.0"
components = ["rustfmt", "clippy"]
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
]
```

### bmad-types/Cargo.toml

```toml
[package]
name = "bmad-types"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
```

### bmad-converter/Cargo.toml

```toml
[package]
name = "bmad-converter"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[lib]
name = "bmad_converter_lib"
path = "src/lib.rs"

[[bin]]
name = "bmad-converter"
path = "src/main.rs"

[dependencies]
bmad-types = { path = "../bmad-types" }
anyhow = { workspace = true }
serde = { workspace = true }
serde_yaml = { workspace = true }
yaml-front-matter = { workspace = true }
quote = { workspace = true }
proc-macro2 = { workspace = true }
clap = { workspace = true }
```

### bmad-plugin/Cargo.toml

```toml
[package]
name = "bmad-plugin"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[lib]
name = "bmad_plugin"
crate-type = ["cdylib"]

[dependencies]
bmad-types = { path = "../bmad-types" }
thiserror = { workspace = true }
# pulse-api = { workspace = true }  # Uncomment when available
```

### .gitignore Content

```
/target
/dist
crates/bmad-plugin/src/generated/
.DS_Store
*.swp
*.swo
```

### Skeleton lib.rs for bmad-types

```rust
// crates/bmad-types/src/lib.rs
// Shared type definitions for bmad-method plugin
// Full types added in Story 1.2

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
```

### Skeleton main.rs for bmad-converter

```rust
// crates/bmad-converter/src/main.rs
fn main() {
    println!("bmad-converter v{}", env!("CARGO_PKG_VERSION"));
}

#[cfg(test)]
mod tests {
    #[test]
    fn main_compiles() {
        // Converter binary compiles successfully
    }
}
```

### Skeleton lib.rs for bmad-plugin

```rust
// crates/bmad-plugin/src/lib.rs
// Pulse plugin entry point — full implementation in Story 1.5

pub fn plugin_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_version_is_nonempty() {
        assert!(!plugin_version().is_empty());
    }
}
```

### Project Structure Notes

This story creates the ENTIRE directory skeleton from architecture.md. The final project tree after this story:

```
bmad-pulse-plugin/
├── README.md               ← Create minimal stub
├── Cargo.toml              ← Workspace root (created here)
├── Cargo.lock              ← Auto-generated
├── rust-toolchain.toml     ← Created here
├── .gitignore              ← Created here
├── agents/                 ← Empty dir (populated in Story 2.3+)
├── scripts/                ← Empty dir (populated in Story 1.6)
├── tests/                  ← Empty dir (integration tests added per story)
├── dist/                   ← .gitkeep (populated in Story 4.1)
└── crates/
    ├── bmad-types/
    │   ├── Cargo.toml
    │   └── src/lib.rs      ← Skeleton with 1 test
    ├── bmad-converter/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs      ← Skeleton with 1 test
    │       └── main.rs     ← Skeleton binary
    └── bmad-plugin/
        ├── Cargo.toml      ← crate-type = ["cdylib"]
        └── src/
            └── lib.rs      ← Skeleton with 1 test
```

**Critical:** `crates/bmad-plugin/src/generated/` is NOT created here — it's git-ignored and will be produced by the converter (Story 1.4). Do not create this directory manually.

### Dependency Graph for This Story

```
bmad-plugin → bmad-types
bmad-converter → bmad-types
bmad-types (no internal deps)
```

Path dependencies use `{ path = "../bmad-types" }` — not published crate references.

### Anti-Patterns to Avoid

- Do NOT set `crate-type = ["cdylib", "rlib"]` — `cdylib` alone is sufficient for the plugin
- Do NOT create `Cargo.lock` manually — cargo generates it
- Do NOT put `pulse-api` as a real dependency yet — it may not be published; leave it commented out
- Do NOT create `src/generated/` manually — it's generated at build time and git-ignored

### References

- [Source: architecture.md#Recommended-Crate-Structure] — workspace layout
- [Source: architecture.md#Complete-Project-Directory-Structure] — full file tree
- [Source: architecture.md#Architectural-Boundaries] — crate dependency rules
- [Source: architecture.md#Key-Dependencies] — dependency versions
- [Source: epics.md#Story-1.1] — acceptance criteria
- [Source: prd.md#Technical-Architecture-Considerations] — cdylib format requirement

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6 (anthropic/claude-sonnet-4-6)

### Debug Log References

- rust-toolchain.toml channel updated from "1.82.0" to "1.85.0" — clap_lex v1.1.0 requires edition2024 which was stabilized in Rust 1.85; 1.85.0 toolchain was available on system.

### Completion Notes List

- ✅ Task 1: Workspace root Cargo.toml created with [workspace], resolver="2", [workspace.package], [workspace.dependencies] — all 3 members included, pulse-api left commented out as specified.
- ✅ Task 2: rust-toolchain.toml created with channel="1.85.0" (adjusted from 1.82.0 for clap_lex edition2024 compatibility), components=[rustfmt, clippy], all 4 targets.
- ✅ Task 3: Full directory structure created — crates/bmad-types/src/, crates/bmad-converter/src/, crates/bmad-plugin/src/, agents/, scripts/, tests/, dist/.gitkeep. Code review (2026-03-17): Added .gitkeep to agents/, scripts/, tests/ — empty dirs without .gitkeep are not tracked by git and would be lost on clone.
- ✅ Task 4: bmad-types crate — Cargo.toml with workspace inheritance, lib.rs skeleton with it_works() test passing.
- ✅ Task 5: bmad-converter crate — Cargo.toml with [lib] + [[bin]], lib.rs with placeholder() + test, main.rs with main() + test. Both pass.
- ✅ Task 6: bmad-plugin crate — Cargo.toml with crate-type=["cdylib"], lib.rs with plugin_version() + nonempty test passing.
- ✅ Task 7: .gitignore created with /target, /dist, crates/bmad-plugin/src/generated/, .DS_Store, *.swp, *.swo.
- ✅ Task 8: cargo build --workspace → Finished dev profile, zero errors. cargo test --workspace → 4 tests pass (it_works, placeholder_works, main_compiles, plugin_version_is_nonempty), zero failures.

### File List

- Cargo.toml
- Cargo.lock
- rust-toolchain.toml
- .gitignore
- dist/.gitkeep
- agents/.gitkeep
- scripts/.gitkeep
- tests/.gitkeep
- crates/bmad-types/Cargo.toml
- crates/bmad-types/src/lib.rs
- crates/bmad-converter/Cargo.toml
- crates/bmad-converter/src/lib.rs
- crates/bmad-converter/src/main.rs
- crates/bmad-plugin/Cargo.toml
- crates/bmad-plugin/src/lib.rs

## Change Log

- 2026-03-17: Story 1.1 implemented — Cargo workspace initialized with 3 crates (bmad-types, bmad-converter, bmad-plugin). All skeleton files created. `cargo build --workspace` and `cargo test --workspace` pass with zero errors. Toolchain pinned to 1.85.0 for edition2024 compatibility with clap_lex 1.1.0.
- 2026-03-17: Code review (adversarial) — All ACs verified against actual code. MEDIUM fix: Added `.gitkeep` to `agents/`, `scripts/`, `tests/` directories (empty dirs are not tracked by git without placeholder files). Story status set to done. 1 issue fixed, 2 LOW issues noted (README.md stub missing; bmad-converter/lib.rs lacks header comment — non-blocking for workspace skeleton story).
