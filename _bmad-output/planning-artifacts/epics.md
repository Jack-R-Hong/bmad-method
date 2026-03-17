---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
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
