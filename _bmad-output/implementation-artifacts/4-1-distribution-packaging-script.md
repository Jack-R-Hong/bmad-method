# Story 4.1: Create Distribution Packaging Script

Status: done

## Story

As a plugin maintainer,
I want a script that packages the compiled plugin binaries and metadata into distributable archives,
so that I can produce release artifacts ready for both CLI installation and direct download.

## Acceptance Criteria

**Given** compiled plugin binaries exist in `target/release/` for the current platform
**When** I run `./scripts/package.sh`
**Then** it produces a platform-specific tarball in `dist/` named `bmad-method-{version}-{platform}.tar.gz`

**Given** the tarball is extracted
**When** I inspect its contents
**Then** it contains: the plugin binary (`libbmad_plugin.{so,dylib}`), a `plugin.toml` manifest with `name`, `version`, `api_version`, and `agent_count` fields, and a `README.md`

**Given** the `plugin.toml` manifest
**When** the `api_version` field is read
**Then** it matches `plugin_api::PLUGIN_API_VERSION` used at compile time (ensuring Pulse compatibility can be checked before loading)

**Given** a version is specified via environment variable (e.g., `PLUGIN_VERSION=1.0.0`)
**When** the packaging script runs
**Then** all output filenames and the `plugin.toml` version field reflect the specified version

**Given** `dist/` is in `.gitignore`
**When** I run `git status` after packaging
**Then** the `dist/` directory and its contents are not tracked by git

## Tasks / Subtasks

- [x] **Task 1: Create `scripts/package.sh` shell script** (AC: #1, #2, #4)
  - [x] Add shebang `#!/usr/bin/env bash` and `set -euo pipefail` for strict mode
  - [x] Detect current platform: use `uname -s` (Linux/Darwin) and `uname -m` (x86_64/aarch64) to set `PLATFORM` variable (e.g., `linux-x86_64`, `darwin-aarch64`)
  - [x] Read `PLUGIN_VERSION` from env var; fall back to `cargo metadata --no-deps --format-version 1` + python3 from `bmad-plugin` crate
  - [x] Set output binary name: `libbmad_plugin.so` on Linux, `libbmad_plugin.dylib` on macOS
  - [x] Verify `target/release/${BINARY_NAME}` exists; exit 1 with clear error if missing
  - [x] Create staging directory `dist/staging/bmad-method-${PLUGIN_VERSION}-${PLATFORM}/`
  - [x] Copy binary, `plugin.toml` (generated in this task), and `README.md` into staging dir
  - [x] Create tarball: `tar -czf dist/bmad-method-${PLUGIN_VERSION}-${PLATFORM}.tar.gz -C dist/staging .`
  - [x] Remove staging directory after tarball creation
  - [x] Print success message: `✓ Packaged: dist/bmad-method-${PLUGIN_VERSION}-${PLATFORM}.tar.gz`

- [x] **Task 2: Generate `plugin.toml` manifest at package time** (AC: #2, #3, #4)
  - [x] Within `scripts/package.sh`, generate `plugin.toml` dynamically (not committed to git as a static file)
  - [x] Use TOML format with these exact fields (api_version is TOML integer, not string, matching u32 PLUGIN_API_VERSION)
  - [x] Read `api_version` from compiled `print_api_version` helper binary via `cargo run -p bmad-plugin --bin print_api_version`
  - [x] Added `src/bin/print_api_version.rs` — uses `#[path]` to access `pulse_api_stub::PLUGIN_API_VERSION` directly
  - [x] Added `src/bin/print_agent_count.rs` — calls `bmad_plugin::registry::list_agents().len()`
  - [x] Script fails with exit 1 if either value cannot be read

- [x] **Task 3: Verify tarball contents are correct** (AC: #2)
  - [x] Add tarball content verification in script: `tar -tzf` and assert all 3 required files are present
  - [x] If any required file is missing from the tarball, script exits with code 1 and prints missing file name

- [x] **Task 4: Update `.gitignore` to exclude `dist/`** (AC: #5)
  - [x] Verified `/dist` line exists in root `.gitignore` (confirmed present from Story 1.1)
  - [x] Confirmed `crates/bmad-plugin/src/generated/` is also in `.gitignore`

- [x] **Task 5: Make script executable and document usage** (AC: #1, #4)
  - [x] Set `chmod +x scripts/package.sh`
  - [x] Usage comment block at top of script with env var docs

- [x] **Task 6: Write integration test for packaging script** (AC: #1, #2, #3)
  - [x] Created `tests/packaging_test.sh` that: runs full build, runs package.sh, extracts tarball, verifies all 3 files present, verifies `plugin.toml` has all required fields
  - [x] This test is manual/CI only — not a Rust unit test

## Dev Notes

### Architecture Context

This story implements the packaging layer defined in the architecture document under "Infrastructure & Build Architecture":
> **Artifact Format:** `.tar.gz` with plugin + metadata — Matches Pulse plugin expectations

The script sits at `scripts/package.sh` in the project root per the full directory structure:
```
scripts/
├── build.sh      # Local build script (Story 1.6)
├── convert.sh    # Run converter (Story 1.6)
└── package.sh    # Create distribution tarball ← THIS STORY
```

### Tarball Structure (Exact)

```
bmad-method-{version}-{platform}.tar.gz
└── bmad-method-{version}-{platform}/
    ├── libbmad_plugin.so      (Linux) OR libbmad_plugin.dylib (macOS)
    ├── plugin.toml
    └── README.md
```

### `plugin.toml` Manifest Format (Exact)

```toml
[plugin]
name = "bmad-method"
version = "1.0.0"
api_version = "0.1.0"
agent_count = 12
```

**Critical:** `api_version` must be obtained from the compiled binary (via helper binary), NOT hardcoded in the script. If the `pulse-api` version changes, `api_version` must update automatically.

### Platform Naming Convention

| `uname -s` | `uname -m` | `PLATFORM` string |
|------------|------------|-------------------|
| `Linux` | `x86_64` | `linux-x86_64` |
| `Linux` | `aarch64` | `linux-aarch64` |
| `Darwin` | `x86_64` | `darwin-x86_64` |
| `Darwin` | `arm64` | `darwin-aarch64` |

**Note:** macOS reports `arm64` from `uname -m`, but we normalize to `aarch64` in the platform string for consistency with Rust target naming.

### Helper Binaries in `bmad-plugin`

Add to `crates/bmad-plugin/Cargo.toml`:
```toml
[[bin]]
name = "print_api_version"
path = "src/bin/print_api_version.rs"

[[bin]]
name = "print_agent_count"
path = "src/bin/print_agent_count.rs"
```

`src/bin/print_api_version.rs`:
```rust
fn main() {
    println!("{}", pulse_api::PLUGIN_API_VERSION);
}
```

`src/bin/print_agent_count.rs`:
```rust
use bmad_plugin::registry;

fn main() {
    println!("{}", registry::all_agents().count());
}
```

### `.gitignore` Requirements (Confirm from Story 1.1)

The root `.gitignore` must contain:
```
/target/
/dist/
crates/bmad-plugin/src/generated/
```

### `PLUGIN_VERSION` Env Var Pattern

```bash
# In package.sh
PLUGIN_VERSION="${PLUGIN_VERSION:-$(cargo metadata --no-deps --format-version 1 \
  | python3 -c "import sys,json; pkgs=json.load(sys.stdin)['packages']; \
    print(next(p['version'] for p in pkgs if p['name']=='bmad-plugin'))")}"
```

Alternative (simpler, if `jq` is available):
```bash
PLUGIN_VERSION="${PLUGIN_VERSION:-$(cargo pkgid -p bmad-plugin | cut -d'#' -f2)}"
```

### FRs and NFRs Fulfilled

- **FR29:** Build system can package plugin for distribution
- **NFR13:** Build pipeline produces reproducible artifacts from the same source (same source → same tarball contents given same PLUGIN_VERSION)

### Project Structure Notes

This story creates new files only in `scripts/` and `crates/bmad-plugin/src/bin/`. It does NOT modify:
- Any Rust source in `src/` (except adding helper binaries)
- The `Cargo.toml` workspace structure (only adds `[[bin]]` entries to `bmad-plugin/Cargo.toml`)
- Generated code in `src/generated/`

Depends on: Epic 1 (working plugin binary in `target/release/`) and Story 1.6 (build pipeline).

### References

- **epics.md** lines 638–666: Story 4.1 full AC definition
- **architecture.md** lines 221–227: Infrastructure & Build Architecture decisions
- **architecture.md** lines 338–416: Complete project directory structure including `scripts/` and `dist/`
- **prd.md** lines 333–342: Build Pipeline (Converter) stage descriptions
- **prd.md** lines 284–291: Installation methods table

## Dev Agent Record

### Agent Model Used
claude-sonnet-4-6

### Debug Log References
None — clean implementation, no regressions.

### Completion Notes List
- `api_version` in `plugin.toml` is a TOML integer (`api_version = 1`), not a string — matches `PLUGIN_API_VERSION: u32 = 1` exactly.
- `print_api_version.rs` uses `#[path = "../pulse_api_stub.rs"] #[allow(dead_code)] mod pulse_api_stub` to access the constant without modifying lib.rs.
- Version fallback uses `cargo metadata` + python3 (avoids `jq` dependency, avoids brittle `cargo pkgid` format).
- `cargo build --workspace` and `cargo test --workspace` (90 tests) pass clean with zero warnings.

### File List
- `scripts/package.sh` (new; updated in code review — added EXIT trap for staging cleanup, stderr capture for cargo run failures, integer validation for API_VERSION/AGENT_COUNT, removed redundant mkdir)
- `crates/bmad-plugin/src/bin/print_api_version.rs` (new)
- `crates/bmad-plugin/src/bin/print_agent_count.rs` (new)
- `crates/bmad-plugin/Cargo.toml` (modified — added `[[bin]]` entries)
- `.gitignore` (verified — `/dist` and `crates/bmad-plugin/src/generated/` already present)
- `tests/packaging_test.sh` (new; updated in code review — strengthened plugin.toml assertions to verify actual field values, validating AC#3 api_version is non-zero integer and AC#4 version matches)
