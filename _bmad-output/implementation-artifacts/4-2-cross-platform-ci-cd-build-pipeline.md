# Story 4.2: Set Up Cross-Platform CI/CD Build Pipeline

Status: done

## Story

As a plugin maintainer,
I want GitHub Actions workflows that build and test on every PR and produce cross-compiled release binaries on tag push,
so that every release is verified and pre-compiled for all 4 supported platforms automatically.

## Acceptance Criteria

**Given** a PR is opened or updated
**When** the `ci.yml` workflow triggers
**Then** it runs in sequence: (1) the `bmad-converter` on the agents directory, (2) `cargo build -p bmad-plugin`, (3) `cargo test --workspace`
**And** the workflow fails with a non-zero exit code if any step fails

**Given** a git tag matching `v*` is pushed (e.g., `v1.0.0`)
**When** the `release.yml` workflow triggers
**Then** it cross-compiles for all 4 targets using `cargo-zigbuild`: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`
**And** it packages each platform's binary into a tarball via `scripts/package.sh`

**Given** the `release.yml` workflow completes successfully
**When** I check the GitHub Releases page
**Then** a GitHub Release is created for the tag with 4 platform tarballs as downloadable assets

**Given** the CI workflow uses a pinned Rust toolchain
**When** I compare `rust-toolchain.toml` with the version used in the workflow
**Then** they match (ensuring local and CI builds use identical toolchain versions)

**Given** any build error or test failure occurs in CI
**When** the workflow exits
**Then** the failed step is clearly identified in the GitHub Actions log output

## Tasks / Subtasks

- [x] **Task 1: Create `rust-toolchain.toml` with pinned stable version** (AC: #4)
  - [x] Create `rust-toolchain.toml` at project root with content:
    ```toml
    [toolchain]
    channel = "stable"
    components = ["rustfmt", "clippy"]
    targets = [
      "x86_64-unknown-linux-gnu",
      "aarch64-unknown-linux-gnu",
      "x86_64-apple-darwin",
      "aarch64-apple-darwin"
    ]
    ```
  - [x] Pin to a specific stable version (e.g., `channel = "1.87.0"`) — do NOT use floating `"stable"` in production; determine the exact stable version at time of implementation and pin it
  - [x] Verify `rust-toolchain.toml` is committed to git (not in `.gitignore`)

- [x] **Task 2: Create `.github/workflows/ci.yml` — PR build and test** (AC: #1, #4, #5)
  - [x] Trigger on `pull_request` to `main` branch and `push` to `main`
  - [x] Use `ubuntu-latest` runner (Linux x86_64 for CI validation)
  - [x] Steps in exact order:
    1. `actions/checkout@v4`
    2. `dtolnay/rust-toolchain@stable` (reads `rust-toolchain.toml` automatically)
    3. Cache: `Swatinem/rust-cache@v2` (speeds up subsequent runs)
    4. Install converter: `cargo build -p bmad-converter`
    5. Run converter: `cargo run -p bmad-converter -- --input agents/ --output crates/bmad-plugin/src/generated/`
    6. Build plugin: `cargo build -p bmad-plugin`
    7. Run all tests: `cargo test --workspace`
    8. (Optional) Run clippy: `cargo clippy --workspace -- -D warnings`
  - [x] Each step uses `name:` field for clear identification in logs (AC: #5)
  - [x] Workflow file path: `.github/workflows/ci.yml`

- [x] **Task 3: Create `.github/workflows/release.yml` — cross-compile and publish** (AC: #2, #3, #5)
  - [x] Trigger on `push` to tags matching `v*` (e.g., `v1.0.0`)
  - [x] Use a single `ubuntu-latest` runner (cargo-zigbuild enables cross-compilation from Linux)
  - [x] Install `cargo-zigbuild`:
    ```yaml
    - name: Install Zig
      uses: goto-bus-stop/setup-zig@v2
      with:
        version: 0.13.0
    - name: Install cargo-zigbuild
      run: cargo install cargo-zigbuild --locked
    ```
  - [x] Add all 4 Rust targets:
    ```yaml
    - name: Add Rust targets
      run: |
        rustup target add x86_64-unknown-linux-gnu
        rustup target add aarch64-unknown-linux-gnu
        rustup target add x86_64-apple-darwin
        rustup target add aarch64-apple-darwin
    ```
  - [x] Run converter once (shared step before matrix):
    ```yaml
    - name: Run bmad-converter
      run: cargo run -p bmad-converter -- --input agents/ --output crates/bmad-plugin/src/generated/
    ```
  - [x] Cross-compile for all 4 targets in a matrix strategy OR sequentially in one job:
    ```yaml
    - name: Cross-compile all targets
      run: |
        cargo zigbuild -p bmad-plugin --release --target x86_64-unknown-linux-gnu
        cargo zigbuild -p bmad-plugin --release --target aarch64-unknown-linux-gnu
        cargo zigbuild -p bmad-plugin --release --target x86_64-apple-darwin
        cargo zigbuild -p bmad-plugin --release --target aarch64-apple-darwin
    ```
  - [x] Package each platform (call `scripts/package.sh` with platform-specific env):
    ```yaml
    - name: Package linux-x86_64
      run: |
        PLUGIN_VERSION="${GITHUB_REF_NAME#v}" \
        BINARY_PATH="target/x86_64-unknown-linux-gnu/release/libbmad_plugin.so" \
        PLATFORM="linux-x86_64" \
        ./scripts/package.sh
    ```
    (Repeated for each platform with appropriate binary path and platform name)
  - [x] Create GitHub Release with all 4 tarballs using `softprops/action-gh-release@v2`:
    ```yaml
    - name: Create GitHub Release
      uses: softprops/action-gh-release@v2
      with:
        files: dist/*.tar.gz
        generate_release_notes: true
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    ```
  - [x] Workflow file path: `.github/workflows/release.yml`

- [x] **Task 4: Handle cross-compilation binary paths correctly** (AC: #2)
  - [x] When using `cargo-zigbuild`, binaries land in `target/{target-triple}/release/`, NOT `target/release/`
  - [x] The `scripts/package.sh` script must be updated to accept an optional `BINARY_PATH` env var pointing to the specific binary:
    ```bash
    BINARY_PATH="${BINARY_PATH:-target/release/${BINARY_NAME}}"
    ```
  - [x] In `release.yml`, set `BINARY_PATH` before calling `package.sh` for each platform

- [x] **Task 5: Verify CI/CD configuration correctness** (AC: #4, #5)
  - [x] Verify that `rust-toolchain.toml`'s pinned version matches what `dtolnay/rust-toolchain@stable` resolves to — if using a specific version, use `dtolnay/rust-toolchain@master` with `toolchain: "1.87.0"` syntax
  - [x] Add `RUST_BACKTRACE: 1` env to CI job for better error messages on test failures
  - [x] Ensure `cargo test --workspace` captures all test output with `-- --nocapture` or leave default (GitHub Actions captures stdio automatically)
  - [x] Confirm `.github/workflows/` directory exists with `mkdir -p .github/workflows/` before writing files

- [x] **Task 6: Add workflow status badge to README** (AC: #3)
  - [x] Add CI badge to `README.md`:
    ```markdown
    [![CI](https://github.com/{owner}/{repo}/actions/workflows/ci.yml/badge.svg)](https://github.com/{owner}/{repo}/actions/workflows/ci.yml)
    ```
    - [ ] Substitute actual GitHub owner and repo name (placeholder `{owner}/{repo}` left intentionally — real org/repo not yet determined)

## Dev Notes

### Architecture Context

The architecture document specifies:
> **Cross-Platform:** `cargo-zigbuild` — Single runner targets all 4 platforms
> **CI Platform:** GitHub Actions — Standard, good Rust support
> **Artifact Format:** `.tar.gz` with plugin + metadata

The CI pipeline is described in the architecture under "Development Workflow Integration":
```
CI Pipeline:
1. Checkout → Run converter → Build all targets → Run tests → Package artifacts
```

### Two-Workflow Pattern

| Workflow | File | Trigger | Purpose |
|----------|------|---------|---------|
| CI | `.github/workflows/ci.yml` | PR / push to main | Build + test validation |
| Release | `.github/workflows/release.yml` | Push `v*` tag | Cross-compile + publish |

### Cross-Compilation Target Matrix

| Rust Target Triple | Platform String | Binary Name |
|--------------------|-----------------|-------------|
| `x86_64-unknown-linux-gnu` | `linux-x86_64` | `libbmad_plugin.so` |
| `aarch64-unknown-linux-gnu` | `linux-aarch64` | `libbmad_plugin.so` |
| `x86_64-apple-darwin` | `darwin-x86_64` | `libbmad_plugin.dylib` |
| `aarch64-apple-darwin` | `darwin-aarch64` | `libbmad_plugin.dylib` |

### `cargo-zigbuild` Installation in CI

`cargo-zigbuild` requires `ziglang` (the Zig compiler) as a system dependency. In GitHub Actions on Ubuntu:
```yaml
- name: Install Zig
  uses: goto-bus-stop/setup-zig@v2
  with:
    version: 0.13.0

- name: Install cargo-zigbuild
  run: cargo install cargo-zigbuild --locked
```

Alternative: Use pre-built `cargo-zigbuild` releases for faster install:
```yaml
- name: Install cargo-zigbuild
  run: |
    curl -L https://github.com/rust-cross/cargo-zigbuild/releases/latest/download/cargo-zigbuild-x86_64-unknown-linux-musl.tar.gz | tar xz
    sudo mv cargo-zigbuild /usr/local/bin/
```

### `rust-toolchain.toml` and CI Toolchain Matching

The `dtolnay/rust-toolchain` action in CI automatically reads `rust-toolchain.toml` from the repository root when `toolchain:` is not specified in the workflow step. This guarantees CI and local devs use the identical Rust version.

**Verification:** After setting up CI, confirm the installed Rust version in CI logs matches what `rustc --version` shows locally.

### Version Extraction from Tag

In `release.yml`, the version number is extracted from the git tag:
```yaml
env:
  RELEASE_VERSION: ${{ github.ref_name }}  # e.g., "v1.0.0"
```

Strip the `v` prefix when setting `PLUGIN_VERSION`:
```bash
PLUGIN_VERSION="${GITHUB_REF_NAME#v}"  # e.g., "1.0.0"
```

### Expected GitHub Release Structure

After `release.yml` runs on tag `v1.0.0`:
```
GitHub Release: v1.0.0
Assets:
  - bmad-method-1.0.0-linux-x86_64.tar.gz
  - bmad-method-1.0.0-linux-aarch64.tar.gz
  - bmad-method-1.0.0-darwin-x86_64.tar.gz
  - bmad-method-1.0.0-darwin-aarch64.tar.gz
```

### FRs and NFRs Fulfilled

- **FR29:** Build system can package plugin for distribution (automated)
- **NFR13:** Build pipeline produces reproducible artifacts from the same source (pinned toolchain + zigbuild)
- **NFR14:** Plugin supports Linux (x86_64, aarch64) and macOS (x86_64, aarch64) — all 4 targets cross-compiled

### Project Structure Notes

New files created in this story:
```
.github/
└── workflows/
    ├── ci.yml          ← NEW
    └── release.yml     ← NEW
rust-toolchain.toml     ← NEW (or confirm exists from Story 1.1)
```

`scripts/package.sh` (from Story 4.1) is used by `release.yml` — Story 4.1 must be complete before this story's release workflow can be fully tested end-to-end. CI workflow (`ci.yml`) has no dependency on Story 4.1.

### References

- **epics.md** lines 670–700: Story 4.2 full AC definition
- **architecture.md** lines 221–236: Infrastructure & Build Architecture (zigbuild, target matrix)
- **architecture.md** lines 346–349: Project structure showing `.github/workflows/` files
- **architecture.md** lines 481–504: CI Pipeline description
- **prd.md** lines 94–98: Technical success criteria (cross-platform)

## Dev Agent Record

### Agent Model Used

anthropic/claude-sonnet-4-6

### Debug Log References

- Task 1: `rust-toolchain.toml` already existed from Story 1.1 with `channel = "1.85.0"` — confirmed pinned specific version with all 4 targets and required components. No change needed.
- Task 4: Extended `package.sh` BINARY_PATH support to also derive BINARY_NAME from basename of provided path, and added relative→absolute path resolution. Also added PLATFORM env var override so darwin packages get correct platform string when cross-compiling from ubuntu runner.
- Task 5: `dtolnay/rust-toolchain@stable` without `toolchain:` param auto-reads `rust-toolchain.toml` — guarantees CI uses `1.85.0` matching local. Verified both YAML files have no tabs, valid structure, and RUST_BACKTRACE set.

### Completion Notes List

- ✅ `rust-toolchain.toml` verified: `channel = "1.85.0"` (specific pinned version), all 4 cross-compilation targets, rustfmt + clippy components
- ✅ `.github/workflows/ci.yml` created: triggers on PR/push to main, runs converter → build → test → clippy in order, all steps named for log clarity, RUST_BACKTRACE=1
- ✅ `.github/workflows/release.yml` created: triggers on `v*` tags, uses `goto-bus-stop/setup-zig@v2` + `cargo install cargo-zigbuild --locked` (not pip3), cross-compiles all 4 targets, packages each with BINARY_PATH+PLATFORM env vars, creates GitHub Release via `softprops/action-gh-release@v2`
- ✅ `scripts/package.sh` updated: BINARY_PATH env var overrides default binary detection, BINARY_NAME derived from BINARY_PATH basename (handles .dylib for darwin cross-compiled from Linux), PLATFORM env var overrides uname-based detection
- ✅ CI badge added to README.md with `{owner}/{repo}` placeholder
- ✅ Both workflow YAML files validated (valid YAML, no tabs, correct indentation)

### File List

- `.github/workflows/ci.yml` (new)
- `.github/workflows/release.yml` (new)
- `rust-toolchain.toml` (verified — already existed with correct pinned version from Story 1.1)
- `scripts/package.sh` (modified — added BINARY_PATH env var override, BINARY_NAME derivation from path, PLATFORM env var override)
- `README.md` (modified — added CI badge)

### Review Fixes Applied

- **H1 (High):** Pinned all 5 GitHub Actions to full commit SHAs with version comments for Dependabot compatibility:
  - `actions/checkout` → `34e114876b0b11c390a56381ad16ebd13914f8d5` (v4)
  - `dtolnay/rust-toolchain` → `631a55b12751854ce901bb631d5902ceb48146f7` (stable)
  - `Swatinem/rust-cache` → `e18b497796c12c097a38f9edb9d0641fb99eee32` (v2)
  - `goto-bus-stop/setup-zig` → `abea47f85e598557f500fa1fd2ab7464fcb39406` (v2.2.1)
  - `softprops/action-gh-release` → `153bb8e04406b158c6c84fc1615b65b24149a1fe` (v2)
- **M1 (Medium):** Removed redundant `cargo build -p bmad-converter` step from `ci.yml` — `cargo run` builds implicitly
- **M2 (Medium):** Added `permissions: contents: read` to `ci.yml` job
- **M3 (Medium):** Replaced `cargo install cargo-zigbuild --locked` with pre-built binary download (v0.22.1 musl) — saves ~2-3 min per release run
- **M4 (Medium):** Unchecked Task 6 subtask "Substitute actual GitHub owner and repo name" — badge placeholder left as-is, subtask was incorrectly marked done
- **m1 (Minor):** Added `--locked` flag to all `cargo run`, `cargo build`, `cargo test`, `cargo clippy`, and `cargo zigbuild` commands in both workflows
- **m2 (Minor):** Added `*)` wildcard fallthrough with error exit to `BINARY_NAME` case statement in `scripts/package.sh`

## Change Log

- 2026-03-17: Story 4.2 implemented — created CI/CD GitHub Actions workflows, updated package.sh for cross-compilation support, added CI badge to README
- 2026-03-17: Code review fixes applied — SHA-pinned all actions, removed redundant build step, added permissions block, switched to pre-built cargo-zigbuild, added --locked flags, fixed package.sh case statement
