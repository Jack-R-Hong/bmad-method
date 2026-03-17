---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-02b-vision
  - step-02c-executive-summary
  - step-03-success
  - step-04-journeys
  - step-05-domain-skipped
  - step-06-innovation-skipped
  - step-07-project-type
  - step-08-scoping
  - step-09-functional
  - step-10-nonfunctional
  - step-11-polish
inputDocuments:
  - /home/jack/Document/pulse/docs/project-overview.md
  - /home/jack/Document/pulse/docs/deep-dive-plugin-system.md
  - /home/jack/Document/pulse/docs/plugin-development-guide.md
  - /home/jack/Document/pulse/docs/index.md
workflowType: 'prd'
documentCounts:
  briefs: 0
  research: 0
  projectDocs: 4
classification:
  projectType: developer_tool
  domain: general
  complexity: medium
  projectContext: brownfield
  primaryUser: Pulse users wanting BMAD-METHOD agents
  keyDecisions:
    converterRole: Build-time internal tool (not user-facing)
    pluginDelivery: Pre-compiled .so/.wasm artifact
    userExperience: Pulse-native installation, no BMAD knowledge required
    installation: Both CLI and manual drop-in
---

# Product Requirements Document - BMAD-METHOD Pulse Plugin

**Author:** Jack
**Date:** 2026-03-17

## Executive Summary

**BMAD-METHOD Pulse Plugin** brings a complete library of specialized AI agents to the Pulse workflow orchestration engine. Pulse provides powerful DAG-based task execution with 7 execution modes and a robust plugin architecture — but ships without pre-built agents. This plugin fills that gap by delivering 12+ battle-tested BMAD-METHOD agents (Architect, Developer, PM, QA, UX, Scrum Master, and more) as a single installable package.

**Target Users:** Pulse users who need ready-to-use AI agents for their workflows without building agent logic from scratch.

**Core Value:** Install once, get a full team of AI specialists ready to execute in Pulse workflows. Zero configuration, Pulse-native experience.

### What Makes This Special

**Instant Productivity.** Pulse users go from "I need AI agents" to "I have specialized experts" with a single command (`pulse plugin install bmad-method`). No agent development, no configuration — just install and orchestrate.

**Battle-Tested Methodology.** BMAD-METHOD agents aren't generic LLM wrappers. Each agent has a distinct persona, communication style, decision-making principles, and domain expertise refined through real-world usage. Winston the Architect thinks differently than Amelia the Developer — and that specialization matters.

**Seamless Integration.** The plugin ships as a pre-compiled artifact (native `.so`/`.dylib` or WASM). A build-time converter transforms BMAD's markdown-based agent definitions into Pulse-compatible format. Users never see the conversion — they just get working agents.

### Project Classification

| Attribute | Value |
|-----------|-------|
| **Project Type** | Developer Tool (Pulse Plugin) |
| **Domain** | DevTools / AI Workflow Orchestration |
| **Complexity** | Medium |
| **Project Context** | Brownfield (integrating with existing Pulse + BMAD-METHOD) |
| **Primary User** | Pulse users wanting pre-built AI agents |
| **Delivery Model** | Pre-compiled plugin + CLI installation |

## Success Criteria

### User Success

| Criterion | Definition |
|-----------|------------|
| **It Works** | Plugin installs successfully and agents execute in Pulse workflows |
| **Discoverability** | Users can find and reference BMAD agents in their workflow definitions |
| **Reliability** | Agent execution completes without errors under normal usage |

### Business Success

| Timeframe | Target |
|-----------|--------|
| **MVP** | Plugin published and installable via both methods (CLI + manual) |
| **3 Months** | Stable release, positive user feedback, no critical bugs |
| **12 Months** | Community adoption, potential for additional agent packs |

### Technical Success

| Criterion | Target |
|-----------|--------|
| **Pulse Compatibility** | Works with Pulse v0.9+ |
| **Plugin Formats** | Native (`.so`/`.dylib`) + WASM support |
| **Agent Coverage** | All 12+ core BMAD agents converted and functional |
| **Converter** | Transforms BMAD `.md` format → Pulse plugin format reliably |
| **Build Pipeline** | Automated build produces distributable artifacts |

### Measurable Outcomes

- [ ] `pulse plugin install bmad-method` completes without errors
- [ ] All 12+ agents loadable and executable in Pulse workflows
- [ ] Plugin passes Pulse's plugin validation checks
- [ ] Manual installation (drop files) works on Linux/macOS/Windows

## User Journeys

### Journey 1: Alex the Pulse Developer — First Installation

**Persona:** Alex, a backend engineer at a startup. Uses Pulse to orchestrate AI-assisted code review and documentation workflows. Tired of writing custom agent prompts from scratch.

**Opening Scene:**
Alex is building a new Pulse workflow for architecture review. They need an AI agent that thinks like a senior architect — not just a generic LLM call. They've heard BMAD-METHOD has battle-tested agent personas.

**Rising Action:**
```bash
# Alex discovers the plugin
pulse plugin search bmad

# Finds it, installs with one command
pulse plugin install bmad-method

# Installation completes
✓ bmad-method v1.0.0 installed
✓ 12 agents available: architect, dev, pm, qa, ux, sm...
```

**Climax:**
Alex opens their workflow file and references the architect agent:
```yaml
steps:
  - name: architecture-review
    executor: bmad/architect
    input: "Review this microservice design..."
```

The workflow executes. Winston the Architect responds with structured architectural feedback — not generic advice, but opinionated guidance grounded in real principles.

**Resolution:**
Alex realizes they just saved hours of prompt engineering. They start using `bmad/dev` for code implementation steps and `bmad/qa` for test generation. The plugin becomes a standard part of their workflow toolkit.

### Journey 2: Alex the Pulse Developer — Using Multiple Agents

**Opening Scene:**
Alex now wants to create a full development workflow: design → implement → test → review.

**Rising Action:**
Alex builds a DAG workflow using multiple BMAD agents:
```yaml
workflow:
  name: feature-development
  steps:
    - name: design
      executor: bmad/architect
      
    - name: implement
      executor: bmad/dev
      depends_on: [design]
      
    - name: test
      executor: bmad/qa
      depends_on: [implement]
      
    - name: review
      executor: bmad/pm
      depends_on: [test]
```

**Climax:**
The workflow executes end-to-end. Each agent brings its personality:
- Winston (Architect) focuses on "what should be"
- Amelia (Dev) is ultra-succinct, references file paths
- Quinn (QA) is practical, "ship it and iterate"
- John (PM) asks "WHY?" and validates user value

**Resolution:**
Alex has a repeatable development workflow powered by a full team of AI specialists. No prompt engineering, no custom agents — just install and orchestrate.

### Journey 3: Sam the DevOps Admin — Plugin Management

**Persona:** Sam, a DevOps engineer who manages the team's Pulse infrastructure.

**Opening Scene:**
Sam's team wants to use BMAD-METHOD agents. Sam needs to ensure the plugin is safe, stable, and maintainable.

**Rising Action:**
```bash
# Sam reviews the plugin before installation
pulse plugin info bmad-method
# Shows: version, size, capabilities, security profile

# Installs in staging first
pulse plugin install bmad-method --env staging

# Runs validation
pulse plugin validate bmad-method
# ✓ All 12 agents load successfully
# ✓ No security warnings
# ✓ Compatible with Pulse v0.9.2
```

**Climax:**
Sam promotes to production:
```bash
pulse plugin install bmad-method --env production
```

**Resolution:**
Sam adds the plugin to their infrastructure-as-code. Future team members get BMAD agents automatically. Plugin updates follow the standard `pulse plugin update` workflow.

### Journey 4: Manual Installation (Alternative Path)

**Opening Scene:**
A developer at a security-conscious enterprise can't use CLI installation due to network restrictions.

**Rising Action:**
1. Download `bmad-method-v1.0.0.tar.gz` from release page
2. Extract to `~/.pulse/plugins/bmad-method/`
3. Restart Pulse or run `pulse plugin reload`

**Climax:**
```bash
pulse plugin list
# ✓ bmad-method (manual) - 12 agents
```

**Resolution:**
Same functionality as CLI installation. Enterprise users have a viable deployment path.

### Journey Requirements Summary

| Journey | Capabilities Revealed |
|---------|----------------------|
| **First Installation** | CLI install command, agent discovery, workflow integration |
| **Multiple Agents** | Agent namespace (`bmad/*`), DAG compatibility, distinct personas |
| **Admin Management** | Plugin info/validate commands, staging support, IaC compatibility |
| **Manual Installation** | File-based install, plugin reload, offline deployment |

## Developer Tool Specific Requirements

### Developer Tool Overview

This is a **Pulse Plugin** — a developer tool that extends the Pulse AI workflow orchestration engine with pre-built BMAD-METHOD agents.

| Attribute | Value |
|-----------|-------|
| **Tool Type** | Runtime plugin (loaded by Pulse engine) |
| **Integration Point** | Pulse plugin API (`TaskExecutor` trait) |
| **Distribution** | Pre-compiled binary + source code |

### Technical Architecture Considerations

#### Plugin Architecture

| Component | Technology | Notes |
|-----------|------------|-------|
| **Plugin Format** | Native (`.so`/`.dylib`) | Primary delivery |
| **WASM Support** | `.wasm` | Stretch goal for sandboxed environments |
| **Language** | Rust | Matches Pulse codebase |
| **API Version** | Pulse Plugin API v0.1.x | Must track Pulse compatibility |

#### Agent Registration

```rust
#[no_mangle]
pub unsafe extern "C" fn pulse_plugin_register() -> *mut PluginRegistration {
    let metadata = PluginMetadata::new(
        "bmad-method",
        "1.0.0",
        plugin_api::PLUGIN_API_VERSION,
    );
    
    let registration = PluginRegistration::new(metadata)
        .with_task_executor(Box::new(BmadArchitect))
        .with_task_executor(Box::new(BmadDev))
        .with_task_executor(Box::new(BmadPM))
        // ... all 12+ agents
        ;
    
    Box::into_raw(Box::new(registration))
}
```

### Installation Methods

| Method | Command/Action | Use Case |
|--------|----------------|----------|
| **CLI Install** | `pulse plugin install bmad-method` | Primary — most users |
| **Manual Install** | Download + extract to `~/.pulse/plugins/` | Enterprise/offline |
| **Source Build** | `cargo build --release` | Contributors/customization |

### Agent API Surface

Each BMAD agent exposes:

| Interface | Description |
|-----------|-------------|
| **Executor Name** | `bmad/{agent}` (e.g., `bmad/architect`, `bmad/dev`) |
| **Input** | Task context + user prompt |
| **Output** | Structured response per agent persona |
| **Configuration** | Optional: model override, temperature, etc. |

### Code Examples

**Basic Usage:**
```yaml
# workflow.yaml
steps:
  - name: review-architecture
    executor: bmad/architect
    input: |
      Review this API design for scalability concerns:
      {{ context.api_spec }}
```

**Multi-Agent Workflow:**
```yaml
steps:
  - name: design
    executor: bmad/architect
    
  - name: implement  
    executor: bmad/dev
    depends_on: [design]
    
  - name: test
    executor: bmad/qa
    depends_on: [implement]
```

### Implementation Considerations

#### Build Pipeline (Converter)

| Stage | Input | Output |
|-------|-------|--------|
| **Parse** | BMAD `.md` files (frontmatter + content) | Agent metadata structs |
| **Transform** | Agent metadata | Rust code (TaskExecutor impls) |
| **Compile** | Rust code | Native `.so`/`.dylib` or `.wasm` |
| **Package** | Compiled artifacts | Distributable tarball |

#### Agent Metadata Extraction

From each BMAD agent `.md` file, extract:
- `name`, `displayName`, `title`, `icon`
- `role`, `identity`, `communicationStyle`, `principles`
- `capabilities` (from frontmatter)

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Problem-Solving MVP
> Ship the smallest thing that solves the core problem: "Pulse users need ready-to-use AI agents."

**MVP Validation Question:** Can a Pulse user install the plugin and execute a BMAD agent in their workflow successfully?

**Resource Requirements:**
- 1 Rust developer (familiar with Pulse plugin architecture)
- Build pipeline setup (CI/CD for plugin compilation)
- ~2-4 weeks for MVP

### MVP Feature Set (Phase 1)

**Core User Journeys Supported:**
- ✓ Journey 1: First Installation (CLI)
- ✓ Journey 2: Using Multiple Agents
- ✓ Journey 4: Manual Installation

**Must-Have Capabilities:**

| Capability | Justification |
|------------|---------------|
| All 12+ BMAD agents | Core value — incomplete = useless |
| CLI installation | Primary user path |
| Manual installation | Enterprise/offline users |
| Native plugin format | Required for Pulse compatibility |
| Agent namespace (`bmad/*`) | Discoverability in workflows |
| Build-time converter | Transforms BMAD → Pulse format |

**Explicitly NOT in MVP:**
- WASM plugin format (stretch goal)
- Selective agent installation
- Agent customization/configuration
- Dashboard UI integration
- Workflow templates

### Post-MVP Features

**Phase 2 (Growth) — 1-2 months post-MVP:**
- WASM plugin format for sandboxed environments
- Agent configuration options (model override, temperature)
- Selective installation (`pulse plugin install bmad-method --agents=architect,dev`)
- Basic documentation site

**Phase 3 (Expansion) — 3-6 months post-MVP:**
- BMAD workflows as Pulse workflow templates
- Dashboard UI integration (agent browser)
- Community agent contributions
- PulseMCP marketplace listing

### Risk Mitigation Strategy

| Risk Type | Risk | Mitigation |
|-----------|------|------------|
| **Technical** | BMAD conversational agents → Pulse transactional model mismatch | Start with batch-mode agents; add interactive support later |
| **Technical** | Pulse plugin API changes | Pin to Pulse v0.9.x; track API compatibility |
| **Market** | Low adoption | Validate with 3-5 Pulse users before full release |
| **Resource** | Fewer resources than planned | MVP is already minimal; no further cuts possible |

## Functional Requirements

### Plugin Installation

- **FR1:** Pulse users can install the BMAD-METHOD plugin via CLI command
- **FR2:** Pulse users can install the BMAD-METHOD plugin by manually placing files in the plugin directory
- **FR3:** Pulse users can verify plugin installation status
- **FR4:** Pulse users can view plugin metadata (version, agent count, compatibility)
- **FR5:** Pulse users can uninstall the plugin via CLI command
- **FR6:** Pulse users can update the plugin to a newer version

### Agent Discovery

- **FR7:** Pulse users can list all available BMAD agents after installation
- **FR8:** Pulse users can view details of a specific BMAD agent (name, description, capabilities)
- **FR9:** Pulse users can reference BMAD agents using the `bmad/{agent}` namespace
- **FR10:** Pulse users can discover agent capabilities through documentation

### Workflow Integration

- **FR11:** Pulse users can use BMAD agents as task executors in workflow definitions
- **FR12:** Pulse users can pass input data to BMAD agents within workflow steps
- **FR13:** Pulse users can chain multiple BMAD agents in a DAG workflow
- **FR14:** Pulse users can receive structured output from BMAD agent execution
- **FR15:** System routes agent execution to appropriate BMAD agent based on executor name

### Agent Execution

- **FR16:** BMAD Architect agent can provide architecture review and design guidance
- **FR17:** BMAD Developer agent can execute code implementation tasks
- **FR18:** BMAD PM agent can perform product management and requirements tasks
- **FR19:** BMAD QA agent can generate and execute test-related tasks
- **FR20:** BMAD UX Designer agent can provide UX design guidance
- **FR21:** BMAD Scrum Master agent can assist with agile ceremonies and planning
- **FR22:** BMAD Analyst agent can perform business analysis tasks
- **FR23:** BMAD Tech Writer agent can generate documentation
- **FR24:** All other BMAD agents (Quick Flow, BMad Master, etc.) can execute their specialized tasks

### Build & Converter (Internal)

- **FR25:** Build system can parse BMAD agent definition files (markdown with frontmatter)
- **FR26:** Build system can extract agent metadata (name, persona, principles, capabilities)
- **FR27:** Build system can transform agent definitions into Pulse-compatible code
- **FR28:** Build system can compile plugin into native binary format
- **FR29:** Build system can package plugin for distribution

### Plugin Compatibility

- **FR30:** Plugin can register with Pulse plugin loader using standard registration function
- **FR31:** Plugin can report its API version for compatibility checking
- **FR32:** Plugin can coexist with other Pulse plugins without conflicts

## Non-Functional Requirements

### Performance

| NFR | Requirement |
|-----|-------------|
| **NFR1** | Plugin loads and registers all agents within 5 seconds of Pulse startup |
| **NFR2** | Individual agent execution adds <500ms overhead beyond LLM response time |
| **NFR3** | Plugin memory footprint remains under 50MB during idle state |

### Integration

| NFR | Requirement |
|-----|-------------|
| **NFR4** | Plugin implements Pulse Plugin API v0.1.x specification completely |
| **NFR5** | Plugin passes Pulse's built-in plugin validation checks |
| **NFR6** | Plugin exports the required `pulse_plugin_register` symbol |
| **NFR7** | Plugin supports both native (`.so`/`.dylib`) and manual installation paths |

### Reliability

| NFR | Requirement |
|-----|-------------|
| **NFR8** | Plugin loads successfully in 100% of compatible Pulse installations |
| **NFR9** | Agent execution failures are gracefully handled and reported to Pulse |
| **NFR10** | Plugin does not crash or hang the Pulse engine under any input |

### Maintainability

| NFR | Requirement |
|-----|-------------|
| **NFR11** | Converter can process new/updated BMAD agents without code changes |
| **NFR12** | Adding a new BMAD agent requires only adding the source `.md` file |
| **NFR13** | Build pipeline produces reproducible artifacts from the same source |

### Compatibility

| NFR | Requirement |
|-----|-------------|
| **NFR14** | Plugin supports Linux (x86_64, aarch64) and macOS (x86_64, aarch64) |
| **NFR15** | Plugin remains compatible with Pulse v0.9.x series |
| **NFR16** | Plugin coexists with other installed Pulse plugins without conflicts |
