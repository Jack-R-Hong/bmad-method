---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - /home/jack/Document/pulse-plugins/bmad-method/_bmad-output/planning-artifacts/prd.md
  - /home/jack/Document/pulse-plugins/bmad-method/_bmad-output/planning-artifacts/architecture.md
---

# BMAD-METHOD Pulse Plugin - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for BMAD-METHOD Pulse Plugin, decomposing the requirements from the PRD and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Pulse users can install the BMAD-METHOD plugin via CLI command
FR2: Pulse users can install the BMAD-METHOD plugin by manually placing files in the plugin directory
FR3: Pulse users can verify plugin installation status
FR4: Pulse users can view plugin metadata (version, agent count, compatibility)
FR5: Pulse users can uninstall the plugin via CLI command
FR6: Pulse users can update the plugin to a newer version
FR7: Pulse users can list all available BMAD agents after installation
FR8: Pulse users can view details of a specific BMAD agent (name, description, capabilities)
FR9: Pulse users can reference BMAD agents using the `bmad/{agent}` namespace
FR10: Pulse users can discover agent capabilities through documentation
FR11: Pulse users can use BMAD agents as task executors in workflow definitions
FR12: Pulse users can pass input data to BMAD agents within workflow steps
FR13: Pulse users can chain multiple BMAD agents in a DAG workflow
FR14: Pulse users can receive structured output from BMAD agent execution
FR15: System routes agent execution to appropriate BMAD agent based on executor name
FR16: BMAD Architect agent can provide architecture review and design guidance
FR17: BMAD Developer agent can execute code implementation tasks
FR18: BMAD PM agent can perform product management and requirements tasks
FR19: BMAD QA agent can generate and execute test-related tasks
FR20: BMAD UX Designer agent can provide UX design guidance
FR21: BMAD Scrum Master agent can assist with agile ceremonies and planning
FR22: BMAD Analyst agent can perform business analysis tasks
FR23: BMAD Tech Writer agent can generate documentation
FR24: All other BMAD agents (Quick Flow, BMad Master, etc.) can execute their specialized tasks
FR25: Build system can parse BMAD agent definition files (markdown with frontmatter)
FR26: Build system can extract agent metadata (name, persona, principles, capabilities)
FR27: Build system can transform agent definitions into Pulse-compatible code
FR28: Build system can compile plugin into native binary format
FR29: Build system can package plugin for distribution
FR30: Plugin can register with Pulse plugin loader using standard registration function
FR31: Plugin can report its API version for compatibility checking
FR32: Plugin can coexist with other Pulse plugins without conflicts

### NonFunctional Requirements

NFR1: Plugin loads and registers all agents within 5 seconds of Pulse startup
NFR2: Individual agent execution adds <500ms overhead beyond LLM response time
NFR3: Plugin memory footprint remains under 50MB during idle state
NFR4: Plugin implements Pulse Plugin API v0.1.x specification completely
NFR5: Plugin passes Pulse's built-in plugin validation checks
NFR6: Plugin exports the required `pulse_plugin_register` symbol
NFR7: Plugin supports both native (`.so`/`.dylib`) and manual installation paths
NFR8: Plugin loads successfully in 100% of compatible Pulse installations
NFR9: Agent execution failures are gracefully handled and reported to Pulse
NFR10: Plugin does not crash or hang the Pulse engine under any input
NFR11: Converter can process new/updated BMAD agents without code changes
NFR12: Adding a new BMAD agent requires only adding the source `.md` file
NFR13: Build pipeline produces reproducible artifacts from the same source
NFR14: Plugin supports Linux (x86_64, aarch64) and macOS (x86_64, aarch64)
NFR15: Plugin remains compatible with Pulse v0.9.x series
NFR16: Plugin coexists with other installed Pulse plugins without conflicts

### Additional Requirements

- Cargo Workspace with workspace inheritance — 3-crate structure: `bmad-types`, `bmad-converter`, `bmad-plugin`
- Build-time binary converter approach: BMAD `.md` files → `bmad-converter` → `generated/*.rs` → `cargo build` → `plugin.so`
- Error handling: `anyhow` for converter (context-rich debugging), `thiserror` for plugin (typed stable interface)
- LLM integration: Agent returns prompt only (`AgentOutput` struct with `system_prompt`, `user_context`, `suggested_params`) — Pulse owns LLM execution
- Cross-platform builds via `cargo-zigbuild` targeting 4 platforms: Linux x86_64/aarch64, macOS x86_64/aarch64
- CI/CD via GitHub Actions with matrix builds (ci.yml for PR, release.yml for cross-compile + publish)
- Agent metadata statically embedded using `&'static str` for compile-time data
- Generated code placed in `crates/bmad-plugin/src/generated/` (git-ignored, clean regeneration)
- All shared types defined only in `bmad-types` crate — never duplicate across crates
- Never panic in plugin boundary code — always return `Result`
- Generated file headers must include source file and timestamp
- Use raw string literals `r#"..."#` for complex agent content
- Document exact BMAD frontmatter YAML schema (Architecture gap — early story)
- Verify `pulse-api` TaskExecutor trait signature against actual crate (Architecture gap — early story)

### UX Design Requirements

N/A — No UX Design document (project is a developer tool plugin with no UI).

### FR Coverage Map

| FR | Epic | Description |
|----|------|-------------|
| FR1 | Epic 4 | CLI installation |
| FR2 | Epic 4 | Manual file installation |
| FR3 | Epic 4 | Verify installation status |
| FR4 | Epic 4 | View plugin metadata |
| FR5 | Epic 4 | Uninstall via CLI |
| FR6 | Epic 4 | Update to newer version |
| FR7 | Epic 3 | List available agents |
| FR8 | Epic 3 | View agent details |
| FR9 | Epic 3 | `bmad/{agent}` namespace |
| FR10 | Epic 3 | Discover via documentation |
| FR11 | Epic 2 | Use agents as task executors |
| FR12 | Epic 2 | Pass input data to agents |
| FR13 | Epic 2 | Chain agents in DAG workflow |
| FR14 | Epic 2 | Receive structured output |
| FR15 | Epic 2 | Route by executor name |
| FR16 | Epic 2 | Architect agent execution |
| FR17 | Epic 2 | Developer agent execution |
| FR18 | Epic 2 | PM agent execution |
| FR19 | Epic 2 | QA agent execution |
| FR20 | Epic 2 | UX Designer agent execution |
| FR21 | Epic 2 | Scrum Master agent execution |
| FR22 | Epic 2 | Analyst agent execution |
| FR23 | Epic 2 | Tech Writer agent execution |
| FR24 | Epic 2 | All other agents execution |
| FR25 | Epic 1 | Parse BMAD agent files |
| FR26 | Epic 1 | Extract agent metadata |
| FR27 | Epic 1 | Transform to Pulse-compatible code |
| FR28 | Epic 1 | Compile to native binary |
| FR29 | Epic 4 | Package for distribution |
| FR30 | Epic 1 | Register with Pulse plugin loader |
| FR31 | Epic 1 | Report API version |
| FR32 | Epic 1 | Coexist with other plugins |

## Epic List

### Epic 1: Build Pipeline — From BMAD Definitions to Working Plugin
A developer can run the converter on BMAD agent markdown files and produce a compilable Pulse plugin binary with all agents registered. End-to-end: put `.md` agent files in, get a working `.so`/`.dylib` out.
**FRs covered:** FR25, FR26, FR27, FR28, FR30, FR31, FR32

### Epic 2: Agent Execution — BMAD Agents Deliver Value in Workflows
Pulse users can use BMAD agents as task executors in their workflow definitions, passing input and receiving structured output — with each agent's distinct persona preserved.
**FRs covered:** FR11, FR12, FR13, FR14, FR15, FR16, FR17, FR18, FR19, FR20, FR21, FR22, FR23, FR24

### Epic 3: Discovery & Metadata — Users Understand What's Available
Pulse users can discover, list, and inspect all available BMAD agents and their capabilities through CLI commands and documentation.
**FRs covered:** FR7, FR8, FR9, FR10

### Epic 4: Distribution & Lifecycle — Package, Install, Manage
Pulse users can install the BMAD-METHOD plugin via CLI or manually, verify it works, update to new versions, and uninstall it. Includes cross-platform packaging and CI/CD.
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR6, FR29

---

## Epic 1: Build Pipeline — From BMAD Definitions to Working Plugin

A developer can run the converter on BMAD agent markdown files and produce a compilable Pulse plugin binary with all agents registered. End-to-end: put `.md` agent files in, get a working `.so`/`.dylib` out.

### Story 1.1: Initialize Cargo Workspace Structure

As a developer,
I want a properly configured Cargo workspace with three crates (`bmad-types`, `bmad-converter`, `bmad-plugin`),
So that I can build and test each component independently with shared dependency management.

**Acceptance Criteria:**

**Given** a fresh repository
**When** I run `cargo build --workspace`
**Then** all three crates compile without errors
**And** the workspace root `Cargo.toml` defines `[workspace]` with all three members
**And** `rust-toolchain.toml` pins a stable Rust version
**And** `.gitignore` excludes `target/`, `dist/`, and `crates/bmad-plugin/src/generated/`
**And** each crate has a skeleton `lib.rs` (or `main.rs` for converter) with a single passing unit test

**Given** the workspace exists
**When** I inspect the `bmad-plugin` crate's `Cargo.toml`
**Then** `crate-type = ["cdylib"]` is set for native plugin output
**And** the crate depends on `bmad-types` via workspace path dependency

**Given** any dependency is declared
**When** I inspect the workspace `Cargo.toml`
**Then** shared dependency versions are defined under `[workspace.dependencies]` and inherited by member crates

*Fulfills: Workspace foundation per architecture, NFR13 (reproducible builds)*

---

### Story 1.2: Define Shared Types in bmad-types

As a developer,
I want all shared data structures defined once in `bmad-types`,
So that `bmad-converter` and `bmad-plugin` share identical type definitions without duplication.

**Acceptance Criteria:**

**Given** the `bmad-types` crate exists
**When** I inspect `src/metadata.rs`
**Then** `AgentMetadata` struct is defined with fields: `id: &'static str`, `name: &'static str`, `display_name: &'static str`, `description: &'static str`, `executor_name: &'static str`, and `capabilities: &'static [&'static str]`
**And** all fields use `&'static str` for compile-time embedded data (not owned `String`)

**Given** the `bmad-types` crate exists
**When** I inspect `src/output.rs`
**Then** `AgentOutput` struct contains `system_prompt: String`, `user_context: String`, and `suggested_params: Option<GenerationParams>`
**And** `GenerationParams` contains at minimum `model: Option<String>` and `temperature: Option<f32>`

**Given** the `bmad-types` crate exists
**When** I inspect `src/error.rs`
**Then** `BmadError` enum is defined using `#[derive(thiserror::Error, Debug)]` with variants: `AgentNotFound(String)`, `InvalidInput(String)`, and `ExecutionFailed(String)`
**And** each variant has a human-readable `#[error(...)]` message in lowercase with no trailing punctuation

**Given** `bmad-types` compiles
**When** I run `cargo test -p bmad-types`
**Then** all unit tests pass, including tests that verify `BmadError` display messages match the specified format

*Fulfills: FR26 (metadata extraction types), type definition patterns, NFR9 (typed error handling)*

---

### Story 1.3: Implement BMAD Frontmatter Parser

As a build system developer,
I want the converter to parse BMAD agent markdown files and extract structured metadata,
So that agent definitions can be reliably transformed into Rust code.

**Acceptance Criteria:**

**Given** a BMAD agent `.md` file with YAML frontmatter containing `name`, `displayName`, `description`, `executor`, and `capabilities` fields
**When** the parser processes the file
**Then** it returns an `AgentMetadata`-equivalent struct with all fields correctly populated
**And** no `unwrap()` or `expect()` calls exist in parser code — every failure returns `Result::Err`

**Given** a `.md` file where frontmatter is missing or malformed
**When** the parser processes the file
**Then** it returns `Err` with an `anyhow` error containing the file path and specific failure reason (e.g., "missing required field 'name'")

**Given** a `.md` file with a `capabilities` field containing a YAML list
**When** the parser processes the file
**Then** capabilities are extracted as a `Vec<String>` in the parsed output

**Given** a directory of multiple agent `.md` files
**When** the converter processes the entire directory
**Then** it successfully parses all files and returns a collection with one parsed entry per agent file

**Given** a `.md` file with non-UTF-8 content, missing required fields, or an empty file
**When** the parser processes the file
**Then** it returns a descriptive `Err` — it does not panic under any input

**Given** the converter runs
**When** I run `cargo test -p bmad-converter`
**Then** all parser unit tests pass, including: valid file with all fields, missing frontmatter delimiter, malformed YAML, missing required field, and empty file

*Fulfills: FR25 (parse BMAD files), FR26 (extract metadata), NFR10 (no crashes), NFR11 (extensible — no code changes for new agents)*

---

### Story 1.4: Implement Rust Code Generator

As a build system developer,
I want the converter to generate valid Rust executor files from parsed agent metadata,
So that each BMAD agent becomes a compilable `TaskExecutor` implementation ready to be included in the plugin.

**Acceptance Criteria:**

**Given** an `AgentMetadata`-equivalent struct for a single agent
**When** the code generator processes it
**Then** it produces a `.rs` file in `crates/bmad-plugin/src/generated/` containing a struct with the agent's data
**And** the generated file begins with the standard header: `//! Auto-generated by bmad-converter. DO NOT EDIT.` followed by the source file name and generation timestamp
**And** agent string content uses raw string literals `r#"..."#` to handle special characters safely

**Given** a batch of parsed agent metadata structs
**When** the generator processes all of them
**Then** it produces one `.rs` file per agent plus a `mod.rs` that re-exports all agent modules
**And** the entire `generated/` directory is overwritten cleanly (no stale files from previous runs)

**Given** the generator has run
**When** I run `cargo build -p bmad-plugin`
**Then** the generated code compiles without errors or warnings

**Given** an agent name with hyphens (e.g., `tech-writer`)
**When** the generator produces the Rust identifier
**Then** the struct name is `PascalCase` (e.g., `TechWriter`) and the file is `tech_writer.rs` with `snake_case` naming

**Given** the converter CLI is invoked with `--input agents/ --output crates/bmad-plugin/src/generated/`
**When** it completes
**Then** it prints a summary: agent count processed and output directory path
**And** it exits with code `0` on success and non-zero on any error, with the error printed to stderr

*Fulfills: FR27 (transform to Pulse-compatible code), NFR11 (add agent = add .md file), NFR12 (no code changes needed), NFR13 (reproducible)*

---

### Story 1.5: Implement Plugin Shell with Registration and API Compliance

As the Pulse engine,
I want the plugin to export a valid `pulse_plugin_register` symbol that registers all agents and reports its API version,
So that Pulse can load and validate the plugin at startup within the 5-second budget.

**Acceptance Criteria:**

**Given** the plugin binary is built
**When** Pulse calls `pulse_plugin_register()`
**Then** it returns a valid `PluginRegistration` with all generated agents registered as `TaskExecutor` implementations
**And** the registration includes plugin metadata: name `"bmad-method"`, version from `env!("CARGO_PKG_VERSION")`, and `plugin_api::PLUGIN_API_VERSION`

**Given** the plugin is loaded
**When** Pulse queries the plugin API version
**Then** the plugin reports `plugin_api::PLUGIN_API_VERSION` matching the compiled `pulse-api` dependency version

**Given** the plugin is loaded alongside other Pulse plugins
**When** all plugins are registered
**Then** no `bmad/` executor name conflicts with core Pulse executors or a second plugin instance
**And** the executor namespace prefix `bmad/` is consistent across all registered agents

**Given** any error occurs during plugin initialization
**When** the error is encountered
**Then** the plugin returns a null pointer or error indicator — it does not panic or call `std::process::exit`
**And** no `unwrap()` or `expect()` calls appear in `lib.rs` or `registry.rs`

**Given** the plugin registers agents via an `all_agents()` iterator
**When** I run `cargo test -p bmad-plugin`
**Then** unit tests verify: agent count is ≥ 1 (at least a stub), every registered executor name starts with `bmad/`, and the registration function returns without panicking

*Fulfills: FR30 (registration), FR31 (API version), FR32 (no conflicts), NFR4, NFR6, NFR8, NFR10*

---

### Story 1.6: End-to-End Build Pipeline Integration

As a developer,
I want a single reproducible build sequence that converts all BMAD agent `.md` files and produces a working native plugin binary,
So that the complete build process is documented and usable by any contributor from a clean checkout.

**Acceptance Criteria:**

**Given** the `agents/` directory contains at least 3 sample `.md` agent files
**When** I run `cargo run -p bmad-converter -- --input agents/ --output crates/bmad-plugin/src/generated/` followed by `cargo build -p bmad-plugin --release`
**Then** the build completes without errors
**And** `target/release/libbmad_plugin.{so,dylib}` is produced

**Given** the plugin binary exists
**When** I run `cargo test --workspace`
**Then** all workspace tests pass (unit and integration)
**And** an integration test verifies the plugin binary exports the `pulse_plugin_register` symbol (via `nm`, `objdump`, or `libloading`)

**Given** the `scripts/build.sh` script exists
**When** I run `./scripts/build.sh`
**Then** it runs the converter step before the cargo build step in the correct order
**And** it exits non-zero if either step fails

**Given** the `crates/bmad-plugin/src/generated/` directory
**When** I run `git status` after a build
**Then** the generated files are not tracked by git (listed in `.gitignore`)

**Given** a developer clones the repository with Rust installed
**When** they follow the README build instructions step-by-step
**Then** they produce a working plugin binary without any undocumented prerequisite steps

*Fulfills: FR28 (compile to native binary), NFR13 (reproducible builds)*

---

## Epic 2: Agent Execution — BMAD Agents Deliver Value in Workflows

Pulse users can use BMAD agents as task executors in their workflow definitions, passing input and receiving structured output — with each agent's distinct persona preserved.

### Story 2.1: Implement TaskExecutor Integration and Agent Routing

As a Pulse workflow engine,
I want the bmad-method plugin to correctly implement the `TaskExecutor` trait and route execution to the right agent by executor name,
So that workflow steps using `executor: bmad/{agent}` are dispatched to the correct BMAD agent implementation.

**Acceptance Criteria:**

**Given** the `pulse-api` crate is available as a dependency
**When** I inspect `executor.rs` and `registry.rs`
**Then** they correctly implement the `TaskExecutor` trait signature as defined in the actual `pulse-api` crate
**And** any differences from assumptions in the architecture doc are reconciled and noted in a code comment

**Given** a workflow step with `executor: bmad/architect`
**When** the Pulse engine calls `execute(task)` on the registered plugin
**Then** the call is dispatched to the Architect agent handler via `registry.rs`

**Given** a workflow step with `executor: bmad/unknown-agent`
**When** the executor's `execute()` is called
**Then** it returns `Err(BmadError::AgentNotFound("unknown-agent".to_string()))` without panicking
**And** the error message includes the unrecognized name for debugging

**Given** a workflow step with an empty executor name
**When** `execute()` is called
**Then** it returns `Err(BmadError::InvalidInput(...))` — it does not panic or return an incorrect result

**Given** the plugin is loaded and tests run
**When** `cargo test -p bmad-plugin` executes
**Then** routing tests cover: valid agent name → correct dispatch; unknown agent name → `AgentNotFound` error; empty executor string → `InvalidInput` error

*Fulfills: FR11 (use agents as task executors), FR15 (route by executor name), NFR9 (graceful error handling), NFR10 (no panics)*

---

### Story 2.2: Implement Input Handling and AgentOutput Construction

As a Pulse workflow author,
I want BMAD agents to correctly parse task input and return a structured `AgentOutput`,
So that agent execution produces usable prompt output that Pulse can pass to LLM execution.

**Acceptance Criteria:**

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

*Fulfills: FR12 (pass input data), FR14 (receive structured output), NFR2 (<500ms overhead — no blocking I/O in execute path), NFR10 (no crashes)*

---

### Story 2.3: Create Core Agent Definition Files — Architect, Dev, PM, QA

As a Pulse workflow author,
I want BMAD Architect, Developer, PM, and QA agents available with their full personas,
So that I can use specialized AI expertise for architecture review, code implementation, product management, and testing tasks.

**Acceptance Criteria:**

**Given** `agents/architect.md` exists with valid frontmatter
**When** the converter parses it
**Then** the frontmatter contains: `name: architect`, `displayName: "Winston the Architect"` (or equivalent), `description`, `executor: bmad/architect`, and a non-empty `capabilities` list
**And** the markdown body contains the Architect's full system prompt including role definition, decision-making principles, and communication style

**Given** `agents/developer.md` exists and the converter processes it
**When** a Pulse workflow step uses `executor: bmad/dev`
**Then** the returned `AgentOutput.system_prompt` reflects the Developer agent's ultra-succinct, technically precise persona

**Given** `agents/pm.md` exists and the converter processes it
**When** a Pulse workflow step uses `executor: bmad/pm`
**Then** the returned `AgentOutput.system_prompt` reflects the PM agent's requirement-validating, "WHY?"-asking persona

**Given** `agents/qa.md` exists and the converter processes it
**When** a Pulse workflow step uses `executor: bmad/qa`
**Then** the returned `AgentOutput.system_prompt` reflects the QA agent's practical, test-focused persona

**Given** all four agent `.md` files exist
**When** I run `cargo run -p bmad-converter -- --input agents/ --output crates/bmad-plugin/src/generated/` and then `cargo build -p bmad-plugin`
**Then** the build succeeds with all four agent executors registered and discoverable via `registry.find_agent()`

*Fulfills: FR16 (Architect agent), FR17 (Developer agent), FR18 (PM agent), FR19 (QA agent)*

---

### Story 2.4: Create Remaining Agent Definition Files — UX, SM, Analyst, Tech Writer, and Others

As a Pulse workflow author,
I want UX Designer, Scrum Master, Analyst, Tech Writer, Quick Flow, BMad Master, and any additional BMAD agents available,
So that I have the full suite of 12+ BMAD specialists for comprehensive AI-powered workflows.

**Acceptance Criteria:**

**Given** the following agent `.md` files exist: `ux-designer.md`, `scrum-master.md`, `analyst.md`, `tech-writer.md`, `quick-flow.md`, `bmad-master.md`
**When** the converter processes the `agents/` directory
**Then** all six files are parsed without errors and generate valid `.rs` executor files

**Given** a workflow uses `executor: bmad/ux` (or `bmad/ux-designer`)
**When** the executor runs
**Then** `AgentOutput.system_prompt` reflects the UX Designer's design-focused, user-empathy-driven persona

**Given** a workflow uses `executor: bmad/sm`
**When** the executor runs
**Then** `AgentOutput.system_prompt` reflects the Scrum Master's agile facilitation and ceremony-guiding persona

**Given** all 12+ agent `.md` files are present in `agents/`
**When** I run the full build sequence (converter + `cargo build -p bmad-plugin`)
**Then** the build succeeds and `registry.list_agents()` returns ≥ 12 agents

**Given** the registry iterates all registered agents
**When** executor names are checked
**Then** no two agents share the same `executor_name`
**And** all `executor_name` values follow the pattern `bmad/{identifier}` with lowercase, hyphen-separated identifiers

**Given** any agent's `.md` file is updated with revised persona content
**When** the converter runs and the plugin is rebuilt
**Then** the plugin reflects the updated persona without any hand-edits to generated or plugin source files

*Fulfills: FR20 (UX agent), FR21 (SM agent), FR22 (Analyst agent), FR23 (Tech Writer agent), FR24 (Quick Flow, BMad Master, and other agents), NFR11, NFR12*

---

### Story 2.5: Verify Multi-Agent DAG Workflow Compatibility

As a Pulse workflow author,
I want to chain multiple BMAD agents in a DAG workflow where later steps receive earlier steps' output as input,
So that I can build end-to-end AI-powered development pipelines using different specialist agents in sequence.

**Acceptance Criteria:**

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

*Fulfills: FR13 (chain in DAG workflow), NFR9 (graceful failure reporting), NFR10 (no crashes or hangs)*

---

## Epic 3: Discovery & Metadata — Users Understand What's Available

Pulse users can discover, list, and inspect all available BMAD agents and their capabilities through CLI commands and documentation.

### Story 3.1: Define BMAD Frontmatter Schema and Verify pulse-api Contract

As a developer implementing the plugin,
I want the BMAD frontmatter YAML schema documented and the `pulse-api` TaskExecutor trait signature confirmed against the real crate,
So that the parser and executor are built against verified specifications — not assumptions that could cause late-breaking failures.

**Acceptance Criteria:**

**Given** the `pulse-api` crate source or documentation is accessible
**When** I inspect the actual `TaskExecutor` trait definition
**Then** a `docs/pulse-api-contract.md` file is created documenting: the exact method signatures, input parameter types, return types, and any required trait bounds (e.g., `Send + Sync`)
**And** any differences between the documented assumptions (in `architecture.md`) and the actual trait are noted with resolution decisions

**Given** the BMAD agent `.md` files in `agents/` are examined
**When** all frontmatter fields across all agents are catalogued
**Then** a `docs/bmad-frontmatter-schema.md` is created documenting: all required fields (`name`, `displayName`, `description`, `executor`, `capabilities`), optional fields with their types, and a complete example of valid frontmatter for a new agent

**Given** the schema documentation exists
**When** a new agent `.md` file is created following the documented schema exactly
**Then** the converter parses it without errors (verified by a test that creates a minimal valid `.md` and runs it through the parser)

**Given** the `pulse-api` contract documentation exists
**When** the executor and registry code is reviewed against it
**Then** every implemented method matches the documented signature with no type mismatches

*Fulfills: FR8 (agent detail/capabilities foundation), Architecture gap: "Document exact BMAD frontmatter YAML schema", Architecture gap: "Verify pulse-api TaskExecutor trait signature"*

---

### Story 3.2: Implement Full Agent Metadata Embedding and Registry Queries

As a Pulse CLI user,
I want to list all available BMAD agents and view details of any specific agent by name,
So that I can discover what agents are available and how to reference them in my workflow files.

**Acceptance Criteria:**

**Given** the plugin is loaded
**When** `registry.list_agents()` is called
**Then** it returns all 12+ `AgentMetadata` entries with non-empty values for: `name`, `display_name`, `description`, `executor_name`, and `capabilities`
**And** the list is returned in a deterministic order (alphabetical by `executor_name`)

**Given** a valid executor name `bmad/architect`
**When** `registry.find_agent("bmad/architect")` is called
**Then** it returns `Some(&AgentMetadata)` with all fields populated

**Given** an unknown executor name `bmad/nonexistent`
**When** `registry.find_agent("bmad/nonexistent")` is called
**Then** it returns `None` — not an error, since this is a query operation

**Given** all registered agents are listed
**When** executor names are inspected
**Then** every `executor_name` follows the `bmad/{identifier}` format
**And** all identifiers use lowercase with hyphens for multi-word names (e.g., `bmad/ux-designer`, `bmad/tech-writer`)

**Given** Pulse surfaces executor information from the plugin's registered `TaskExecutor` implementations
**When** a user queries available executors via the Pulse CLI
**Then** all `bmad/` executors appear in the output

**Given** a compile-time assertion or unit test is present
**When** the agent count in the registry is checked
**Then** it matches the number of `.md` files in `agents/` at build time

*Fulfills: FR7 (list available agents), FR8 (view agent details), FR9 (bmad/agent namespace), NFR1 (all agents registered within 5s of startup)*

---

### Story 3.3: Write Comprehensive Agent Reference Documentation

As a Pulse workflow author,
I want comprehensive documentation that lists all BMAD agents, their capabilities, executor names, and workflow YAML examples,
So that I can discover the right agent for my task without reading source code or trial and error.

**Acceptance Criteria:**

**Given** the `README.md` exists in the repository root
**When** a user reads it
**Then** it contains all of: installation instructions (both CLI and manual methods), a complete agent reference table with all 12+ agents showing executor name and one-line description, and at least 2 complete workflow YAML examples

**Given** the agent reference table in `README.md`
**When** compared against the registered agents in the plugin
**Then** every registered `bmad/` executor has a corresponding row in the table
**And** every row in the table corresponds to an actual registered executor (no phantom entries)

**Given** a workflow author wants to use `bmad/architect`
**When** they find it in the README
**Then** they can read: what the Architect agent specializes in, the exact executor name to use, and a minimal workflow YAML snippet showing it in a step definition

**Given** a new agent `.md` file is added to `agents/`
**When** the README agent table is updated
**Then** the update requires only appending one row — no structural changes to the README are needed

**Given** the README contains executor names
**When** every executor name in the documentation is compared against `registry.list_agents()`
**Then** all documented executor names match actual registered names exactly (no typos or outdated names)

*Fulfills: FR10 (discover agent capabilities through documentation)*

---

## Epic 4: Distribution & Lifecycle — Package, Install, Manage

Pulse users can install the BMAD-METHOD plugin via CLI or manually, verify it works, update to new versions, and uninstall it. Includes cross-platform packaging and CI/CD.

### Story 4.1: Create Distribution Packaging Script

As a plugin maintainer,
I want a script that packages the compiled plugin binaries and metadata into distributable archives,
So that I can produce release artifacts ready for both CLI installation and direct download.

**Acceptance Criteria:**

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

*Fulfills: FR29 (package for distribution), NFR13 (reproducible artifacts)*

---

### Story 4.2: Set Up Cross-Platform CI/CD Build Pipeline

As a plugin maintainer,
I want GitHub Actions workflows that build and test on every PR and produce cross-compiled release binaries on tag push,
So that every release is verified and pre-compiled for all 4 supported platforms automatically.

**Acceptance Criteria:**

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

*Fulfills: FR29 (automated packaging), NFR13 (reproducible builds), NFR14 (4-platform support: Linux x86_64/aarch64, macOS x86_64/aarch64)*

---

### Story 4.3: Implement CLI Plugin Installation Support

As a Pulse user,
I want to install the BMAD-METHOD plugin with a single `pulse plugin install bmad-method` command,
So that I can get all 12+ BMAD agents without manual file operations or build steps.

**Acceptance Criteria:**

**Given** the plugin is published to a Pulse-compatible registry or as GitHub Release assets
**When** I run `pulse plugin install bmad-method`
**Then** the plugin binary is downloaded and placed in the correct Pulse plugin directory for the current platform
**And** the command completes with a success message that includes the installed version and agent count

**Given** the installation succeeds
**When** I run `pulse plugin list`
**Then** `bmad-method` appears in the list with its version number

**Given** the network is unavailable during installation
**When** `pulse plugin install bmad-method` is run
**Then** it fails with a clear error message and does not leave partial or corrupted files in the plugin directory

**Given** a version of the plugin is already installed
**When** I run `pulse plugin install bmad-method` again
**Then** the command either reports "already installed" or prompts the user — it does not silently overwrite the existing installation

**Given** the installation completes
**When** a Pulse workflow using `executor: bmad/architect` runs
**Then** the executor is found and produces a valid `AgentOutput` without error

*Fulfills: FR1 (CLI installation), NFR7 (CLI install path), NFR8 (100% load success on compatible Pulse)*

---

### Story 4.4: Implement Manual Installation Support

As an enterprise Pulse user with network restrictions,
I want to install the BMAD-METHOD plugin by manually placing downloaded release files in the correct directory,
So that I can use BMAD agents in air-gapped or network-restricted environments.

**Acceptance Criteria:**

**Given** a user downloads `bmad-method-v1.0.0-linux-x86_64.tar.gz` from the GitHub Releases page
**When** they extract it to `~/.pulse/plugins/bmad-method/`
**Then** the plugin loads correctly when Pulse starts (or on `pulse plugin reload`)
**And** all 12+ `bmad/` executors become available in Pulse workflows

**Given** the manual installation is complete
**When** I run `pulse plugin list`
**Then** `bmad-method` appears with its version and agent count

**Given** the README "Manual Installation" section
**When** a user follows the instructions step-by-step without CLI access
**Then** they can install the plugin successfully without requiring the Pulse CLI install command or internet access beyond downloading the tarball

**Given** the plugin directory after extraction
**When** I inspect `~/.pulse/plugins/bmad-method/`
**Then** the plugin binary and `plugin.toml` manifest are the only files required for Pulse to load the plugin (no additional runtime dependencies)

**Given** the `plugin.toml` `api_version` in the tarball does not match the installed Pulse version
**When** Pulse attempts to load the plugin
**Then** Pulse reports a version compatibility warning (this behavior is Pulse's responsibility; the plugin must provide the correct `api_version` in `plugin.toml`)

*Fulfills: FR2 (manual file installation), NFR7 (supports manual install path)*

---

### Story 4.5: Implement Plugin Verification and Metadata Display

As a DevOps engineer managing Pulse infrastructure,
I want to verify that the bmad-method plugin is correctly installed and view its metadata,
So that I can confirm a healthy installation before deploying workflows to production.

**Acceptance Criteria:**

**Given** the plugin is installed
**When** I run `pulse plugin verify bmad-method` (or the equivalent Pulse verification command)
**Then** it checks that all registered agents load without error
**And** reports ✓ for each passing check and ✗ with a reason for any failure

**Given** the plugin is installed
**When** I run `pulse plugin info bmad-method` (or equivalent)
**Then** it displays: plugin version, API compatibility status (compatible / incompatible with reason), total agent count, and list of all `bmad/` executor names

**Given** a corrupted or partial installation (e.g., binary exists but `plugin.toml` is missing)
**When** the verify command runs
**Then** it reports the specific failure reason (e.g., "manifest file missing", "binary not found", "API version mismatch") — it does not crash or produce an unhelpful error

**Given** the info command displays the agent count
**When** compared against the actual number of registered `TaskExecutor` instances in the plugin
**Then** the displayed count matches exactly

*Fulfills: FR3 (verify installation status), FR4 (view plugin metadata), NFR5 (passes Pulse plugin validation checks)*

---

### Story 4.6: Implement Plugin Uninstall and Update

As a Pulse user,
I want to uninstall or update the BMAD-METHOD plugin using standard Pulse CLI commands,
So that I can manage the plugin lifecycle like any other Pulse plugin without manual file operations.

**Acceptance Criteria:**

**Given** the plugin is installed
**When** I run `pulse plugin uninstall bmad-method`
**Then** the plugin binary and manifest are removed from the plugin directory
**And** subsequent `pulse plugin list` no longer shows `bmad-method`
**And** Pulse starts without attempting to load the removed plugin

**Given** the plugin has been uninstalled
**When** a workflow references `executor: bmad/architect`
**Then** Pulse reports a clear error indicating the executor is unavailable (not a crash)

**Given** an older version of the plugin is installed
**When** I run `pulse plugin update bmad-method`
**Then** the latest release binary replaces the old one and the new version appears in `pulse plugin list`

**Given** an update fails mid-way due to a network error
**When** the update command exits
**Then** the original plugin binary is still present and functional (the update does not leave the plugin in a broken state)

**Given** the plugin is already at the latest available version
**When** I run `pulse plugin update bmad-method`
**Then** it reports "already up to date" and exits cleanly without re-downloading or overwriting the existing binary

*Fulfills: FR5 (uninstall via CLI), FR6 (update to newer version)*
