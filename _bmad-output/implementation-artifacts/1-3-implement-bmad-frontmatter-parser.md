# Story 1.3: Implement BMAD Frontmatter Parser

Status: done

## Story

As a build system developer,
I want the converter to parse BMAD agent markdown files and extract structured metadata,
so that agent definitions can be reliably transformed into Rust code.

## Acceptance Criteria

**AC1:**
**Given** a BMAD agent `.md` file with YAML frontmatter containing `name`, `displayName`, `description`, `executor`, and `capabilities` fields
**When** the parser processes the file
**Then** it returns an `AgentMetadata`-equivalent struct with all fields correctly populated
**And** no `unwrap()` or `expect()` calls exist in parser code — every failure returns `Result::Err`

**AC2:**
**Given** a `.md` file where frontmatter is missing or malformed
**When** the parser processes the file
**Then** it returns `Err` with an `anyhow` error containing the file path and specific failure reason (e.g., "missing required field 'name'")

**AC3:**
**Given** a `.md` file with a `capabilities` field containing a YAML list
**When** the parser processes the file
**Then** capabilities are extracted as a `Vec<String>` in the parsed output

**AC4:**
**Given** a directory of multiple agent `.md` files
**When** the converter processes the entire directory
**Then** it successfully parses all files and returns a collection with one parsed entry per agent file

**AC5:**
**Given** a `.md` file with non-UTF-8 content, missing required fields, or an empty file
**When** the parser processes the file
**Then** it returns a descriptive `Err` — it does not panic under any input

**AC6:**
**Given** the converter runs
**When** I run `cargo test -p bmad-converter`
**Then** all parser unit tests pass, including: valid file with all fields, missing frontmatter delimiter, malformed YAML, missing required field, and empty file

## Tasks / Subtasks

- [x] **Task 1: Create parser module structure** (AC: #1)
  - [x] Create `crates/bmad-converter/src/parser/mod.rs` — pub re-export of frontmatter module
  - [x] Create `crates/bmad-converter/src/parser/frontmatter.rs` — core parsing logic
  - [x] Update `crates/bmad-converter/src/lib.rs` to declare `pub mod parser;`

- [x] **Task 2: Define ParsedAgent intermediate struct** (AC: #1, #3)
  - [x] Define `ParsedAgent` struct in `parser/frontmatter.rs` — owned `String` fields (not `&'static str`)
  - [x] Fields: `name: String`, `display_name: String`, `description: String`, `executor_name: String`, `capabilities: Vec<String>`, `body: String`
  - [x] Derive `Debug`, `Clone` on `ParsedAgent`
  - [x] Note: This is the intermediate representation; `AgentMetadata` (static str) is produced during code generation in Story 1.4

- [x] **Task 3: Define FrontmatterData serde struct** (AC: #1, #3)
  - [x] Define `FrontmatterData` struct matching the YAML schema, derived with `serde::Deserialize`
  - [x] Fields: `name: Option<String>`, `display_name: Option<String>` (serde alias `displayName`), `description: Option<String>`, `executor: Option<String>`, `capabilities: Option<Vec<String>>`
  - [x] Use `#[serde(rename = "displayName")]` or `#[serde(alias = "displayName")]` for camelCase compatibility

- [x] **Task 4: Implement parse_file() function** (AC: #1, #2, #5)
  - [x] Signature: `pub fn parse_file(path: &Path) -> anyhow::Result<ParsedAgent>`
  - [x] Read file content with `std::fs::read_to_string(path)` — wrap error with `.context(format!("failed to read file: {}", path.display()))`
  - [x] Use `yaml_front_matter::YamlFrontMatter::parse::<FrontmatterData>(&content)` to parse
  - [x] Handle missing frontmatter: if parse returns error, return `Err` with path + reason
  - [x] Validate each required field is `Some(...)` — return specific error for first missing field
  - [x] Map to `ParsedAgent` struct
  - [x] Extract markdown body (everything after frontmatter delimiters) into `body` field

- [x] **Task 5: Implement parse_directory() function** (AC: #4)
  - [x] Signature: `pub fn parse_directory(dir: &Path) -> anyhow::Result<Vec<ParsedAgent>>`
  - [x] Use `std::fs::read_dir(dir)` to iterate `.md` files — wrap with context
  - [x] Filter entries by `.md` extension only
  - [x] Call `parse_file()` for each; collect results
  - [x] If ANY file fails, return `Err` with the file path and reason (fail fast)
  - [x] Sort results by `name` field for deterministic ordering

- [x] **Task 6: Write parser unit tests** (AC: #6)
  - [x] Test `parse_file()` with a valid in-memory temp file (all required fields present)
  - [x] Test `parse_file()` with a file missing frontmatter delimiters (`---`)
  - [x] Test `parse_file()` with malformed YAML (`name: [unclosed`)
  - [x] Test `parse_file()` with frontmatter missing required field `name`
  - [x] Test `parse_file()` with an empty file (0 bytes)
  - [x] Test `parse_file()` with a `capabilities` YAML list → verifies `Vec<String>` extraction
  - [x] Test `parse_directory()` with 2 valid agent files

- [x] **Task 7: Verify no unwrap/expect in parser code** (AC: #1)
  - [x] Search all files under `src/parser/` for `unwrap()` and `expect(` — must be zero occurrences
  - [x] Run `cargo clippy -p bmad-converter` — zero warnings

## Dev Notes

### BMAD Frontmatter YAML Schema

Every BMAD agent `.md` file in `agents/` must begin with a YAML frontmatter block between `---` delimiters:

```yaml
---
name: architect
displayName: "Winston the Architect"
description: "Architecture review and design guidance for software systems"
executor: bmad/architect
capabilities:
  - architecture-review
  - system-design
  - technical-guidance
  - trade-off-analysis
---

# Winston the Architect

You are Winston, a senior software architect...
(rest of markdown body is the system prompt)
```

**Required fields:** `name`, `displayName`, `description`, `executor`, `capabilities`
**Field mapping:**
- YAML `name` → `ParsedAgent.name` (also used as `id`)
- YAML `displayName` → `ParsedAgent.display_name`
- YAML `description` → `ParsedAgent.description`
- YAML `executor` → `ParsedAgent.executor_name` (must start with `bmad/`)
- YAML `capabilities` → `ParsedAgent.capabilities: Vec<String>`
- Markdown body (after closing `---`) → `ParsedAgent.body`

### Complete parser/mod.rs

```rust
// crates/bmad-converter/src/parser/mod.rs
pub mod frontmatter;

pub use frontmatter::{parse_directory, parse_file, ParsedAgent};
```

### Complete parser/frontmatter.rs

```rust
// crates/bmad-converter/src/parser/frontmatter.rs
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Serde-compatible struct matching the BMAD agent YAML frontmatter schema.
/// All fields are `Option<_>` to allow graceful missing-field error messages.
#[derive(Debug, Deserialize)]
struct FrontmatterData {
    pub name: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub executor: Option<String>,
    pub capabilities: Option<Vec<String>>,
}

/// Intermediate representation of a parsed BMAD agent file.
/// Uses owned `String` (not `&'static str`) — static embedding happens in codegen (Story 1.4).
#[derive(Debug, Clone)]
pub struct ParsedAgent {
    /// Internal identifier, e.g., "architect"
    pub name: String,
    /// Human-readable display name, e.g., "Winston the Architect"
    pub display_name: String,
    /// One-line description of the agent's purpose
    pub description: String,
    /// Pulse executor name, e.g., "bmad/architect"
    pub executor_name: String,
    /// Capability tags for agent discovery
    pub capabilities: Vec<String>,
    /// Full markdown body (system prompt content, everything after frontmatter)
    pub body: String,
}

/// Parse a single BMAD agent markdown file and return structured metadata.
///
/// # Errors
/// Returns `Err` for: file read failure, missing/malformed frontmatter,
/// missing required fields, or invalid executor format. Never panics.
pub fn parse_file(path: &Path) -> Result<ParsedAgent> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read file: {}", path.display()))?;

    if content.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "file is empty: {}",
            path.display()
        ));
    }

    let document = yaml_front_matter::YamlFrontMatter::parse::<FrontmatterData>(&content)
        .map_err(|e| anyhow::anyhow!(
            "failed to parse frontmatter in {}: {}",
            path.display(),
            e
        ))?;

    let fm = document.metadata;
    let body = document.content.trim().to_string();

    let name = fm.name.ok_or_else(|| anyhow::anyhow!(
        "missing required field 'name' in {}",
        path.display()
    ))?;

    let display_name = fm.display_name.ok_or_else(|| anyhow::anyhow!(
        "missing required field 'displayName' in {}",
        path.display()
    ))?;

    let description = fm.description.ok_or_else(|| anyhow::anyhow!(
        "missing required field 'description' in {}",
        path.display()
    ))?;

    let executor_name = fm.executor.ok_or_else(|| anyhow::anyhow!(
        "missing required field 'executor' in {}",
        path.display()
    ))?;

    let capabilities = fm.capabilities.ok_or_else(|| anyhow::anyhow!(
        "missing required field 'capabilities' in {}",
        path.display()
    ))?;

    Ok(ParsedAgent {
        name,
        display_name,
        description,
        executor_name,
        capabilities,
        body,
    })
}

/// Parse all `.md` files in a directory and return a sorted collection.
///
/// # Errors
/// Returns `Err` if any single file fails to parse. Fails fast.
pub fn parse_directory(dir: &Path) -> Result<Vec<ParsedAgent>> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read directory: {}", dir.display()))?;

    let mut agents = Vec::new();

    for entry in entries {
        let entry = entry.with_context(|| format!("failed to read directory entry in {}", dir.display()))?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let agent = parse_file(&path)
                .with_context(|| format!("failed to parse agent file: {}", path.display()))?;
            agents.push(agent);
        }
    }

    // Deterministic order for reproducible code generation
    agents.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(agents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp_md(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }
    // ... (tests omitted for brevity in Dev Notes)
}
```

### Updated lib.rs for bmad-converter

```rust
// crates/bmad-converter/src/lib.rs
//! Build-time converter: BMAD agent markdown files → Rust code.
//! Exposed as a library for testability; CLI entry point is main.rs.

pub mod parser;

// codegen module added in Story 1.4
```

### Dependency: Add tempfile to bmad-converter for tests

Add to `crates/bmad-converter/Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3.0"
```

Add `tempfile = "3.0"` to `[workspace.dependencies]` as well.

### yaml-front-matter Crate Usage

The `yaml-front-matter` crate (version `0.1`) provides `YamlFrontMatter::parse::<T>(&str)`.

**Return type:** `Result<Document<T>, ...>` where `Document<T>` has:
- `.metadata: T` — the deserialized frontmatter struct
- `.content: String` — everything after the closing `---` delimiter

**Important:** The crate expects `---` delimiters at the start of the file. Files without frontmatter will return an error from `parse()`.

**Serde alias for camelCase:** The BMAD format uses `displayName` (camelCase). Use `#[serde(rename = "displayName")]` on the `display_name` field in `FrontmatterData`.

### Error Handling Pattern (AC: #1, #2, #5)

All errors in the parser use `anyhow`:
- File read errors: `.with_context(|| format!("failed to read file: {}", path.display()))`
- Parse errors: `anyhow::anyhow!("failed to parse frontmatter in {}: {}", path.display(), e)`
- Missing field errors: `anyhow::anyhow!("missing required field 'name' in {}", path.display())`

**Critical:** Zero `unwrap()` or `expect()` anywhere in `src/parser/`. The `#[cfg(test)]` block can use `.unwrap()` in tests where failure means the test should fail, but not in production code paths.

### Project Structure Notes

Files created/modified in this story:

```
crates/bmad-converter/
├── Cargo.toml          ← Add tempfile to [dev-dependencies]
└── src/
    ├── lib.rs          ← Add `pub mod parser;`
    ├── main.rs         ← Unchanged (CLI wiring in Story 1.4)
    └── parser/         ← NEW directory
        ├── mod.rs      ← NEW: pub re-export
        └── frontmatter.rs ← NEW: parse_file(), parse_directory(), ParsedAgent
```

`bmad-types` is not modified in this story. The `ParsedAgent` struct uses `String` (not `&'static str`) because it's an intermediate build-time representation — static embedding happens when the code generator writes the `.rs` files.

### References

- [Source: architecture.md#Complete-Project-Directory-Structure] — `crates/bmad-converter/src/parser/` path
- [Source: architecture.md#Code-Generation-Patterns] — clean regeneration, body as system prompt content
- [Source: architecture.md#Error-Handling-Patterns] — anyhow for converter, context-rich errors
- [Source: architecture.md#Key-Dependencies] — yaml-front-matter crate
- [Source: epics.md#Story-1.3] — all acceptance criteria
- [Source: prd.md#Agent-Metadata-Extraction] — fields to extract: name, displayName, capabilities

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6 (anthropic/claude-sonnet-4-6)

### Debug Log References

None — clean implementation with no issues.

### Completion Notes List

- ✅ Task 1: Created `parser/mod.rs` (pub re-export) and `parser/frontmatter.rs` (core logic); updated `lib.rs` with `pub mod parser;`
- ✅ Task 2: `ParsedAgent` struct defined with owned `String` fields (`name`, `display_name`, `description`, `executor_name`, `capabilities`, `body`); derives `Debug`, `Clone`
- ✅ Task 3: `FrontmatterData` with `#[serde(rename = "displayName")]` for camelCase compatibility; all fields `Option<_>`
- ✅ Task 4: `parse_file()` uses `with_context`/`anyhow::anyhow!` throughout — zero panics
- ✅ Task 5: `parse_directory()` filters `.md` files, fails fast on any error, sorts by `name`
- ✅ Task 6: 7 tests written — valid file, missing delimiter, malformed YAML, missing field, empty file, capabilities vec, directory parsing (2 agents sorted alphabetically)
- ✅ Task 7: Zero `unwrap()`/`expect()` in production code; zero clippy warnings
- Test run: `cargo test -p bmad-converter` — 8 tests pass (7 parser + 1 existing placeholder)
- Added `tempfile = "3.0"` to `[workspace.dependencies]` in root `Cargo.toml` and `[dev-dependencies]` in `bmad-converter/Cargo.toml`

### File List

- `crates/bmad-converter/src/parser/mod.rs` (NEW)
- `crates/bmad-converter/src/parser/frontmatter.rs` (NEW)
- `crates/bmad-converter/src/lib.rs` (MODIFIED — added `pub mod parser;`)
- `crates/bmad-converter/Cargo.toml` (MODIFIED — added `[dev-dependencies] tempfile`)
- `Cargo.toml` (MODIFIED — added `tempfile = "3.0"` to `[workspace.dependencies]`)

## Change Log

- 2026-03-17: Story implemented — parser module created with `parse_file()`, `parse_directory()`, `ParsedAgent`, 7 unit tests; all ACs satisfied; status set to "review"
- 2026-03-17: Code review completed — adversarial review passed: 0 HIGH, 0 MEDIUM issues; all 6 ACs verified against code; all 7 tasks confirmed [x]; zero unwrap()/expect() in production parser code; 8/8 tests pass; status set to "done"
