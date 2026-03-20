# BMAD-METHOD Pulse Plugin

[![CI](https://github.com/{owner}/{repo}/actions/workflows/ci.yml/badge.svg)](https://github.com/{owner}/{repo}/actions/workflows/ci.yml)

A Pulse plugin delivering a full team of battle-tested BMAD-METHOD AI agents for workflow orchestration.
Install once and get specialized AI expertise — Architect, Developer, PM, QA, UX, Security, DevOps, and more.

## Quick Start

```bash
pulse plugin install bmad-method
```

Then reference agents in your workflow:

```yaml
workflow:
  name: my-workflow
  steps:
    - name: review
      executor: bmad/architect
      input: "Review this API design for scalability concerns"
```

## Installation

### CLI Installation (Recommended)

> **Requires Pulse v0.9.0 or later.** Check your version with `pulse --version`.

```bash
pulse plugin install bmad-method
```

Installs the latest version. After installation, all `bmad/` executors are available in Pulse workflows.

Expected output:
```
✓ bmad-method v{version} installed
✓ 12 agents available: architect, dev, pm, qa, ux-designer, sm, quick-flow-solo-dev...
```

#### Verifying Your Installation

Run `pulse plugin list` to confirm the plugin is registered:
```bash
pulse plugin list
# bmad-method v{version} — 12 agents
```

Then run a minimal workflow to confirm the `bmad/architect` executor resolves correctly:
```yaml
# test-bmad.yaml
workflow:
  name: bmad-install-test
  steps:
    - name: test-architect
      executor: bmad/architect
      input: "Say hello from the Architect agent"
```
```bash
pulse run test-bmad.yaml
```
Expected: the step executes and returns a non-empty response from the Architect agent.

For a full automated acceptance test, run the verification script (requires Pulse installed):
```bash
bash tests/cli_install_verification.sh
```

#### If Installation Fails

**Network failure during download:** Pulse may leave a partial plugin directory. If you see load errors after a failed install, remove the partial files and retry:
```bash
rm -rf ~/.pulse/plugins/bmad-method/
pulse plugin install bmad-method
```
A complete network failure leaves no files, but an interrupted connection may leave a partial binary that appears corrupt (invalid ELF/Mach-O header). The `rm -rf` above is the safe recovery path.

**Plugin already installed:** Running `pulse plugin install bmad-method` when the plugin is already present will report the current version or prompt before overwriting:
```
Plugin bmad-method v{version} is already installed. Use `pulse plugin update bmad-method` to upgrade.
```
If your Pulse version silently reinstalls instead of prompting, the existing installation is replaced in-place — this is safe, though `pulse plugin update bmad-method` is the preferred upgrade path.

### Manual Installation

Use this method for air-gapped or network-restricted environments. In air-gap scenarios:
download the tarball on an internet-connected machine, transfer it to the target machine
(USB, secure copy, internal artifact server, etc.), then follow the steps below.

#### 1. Download the Release

Go to [GitHub Releases](https://github.com/<YOUR-ORG>/bmad-method/releases) and download the
tarball for your platform:

> Replace `<YOUR-ORG>` with the actual GitHub organization hosting this plugin.

| Platform | File |
|----------|------|
| Linux x86_64 | `bmad-method-{version}-linux-x86_64.tar.gz` |
| Linux aarch64 | `bmad-method-{version}-linux-aarch64.tar.gz` |
| macOS x86_64 | `bmad-method-{version}-darwin-x86_64.tar.gz` |
| macOS aarch64 (Apple Silicon) | `bmad-method-{version}-darwin-aarch64.tar.gz` |

#### 2. Create Plugin Directory

```bash
mkdir -p ~/.pulse/plugins/bmad-method
```

#### 3. Extract and Copy Files

Replace `{version}` and `{arch}` with the values matching your download — e.g. `1.0.0` and
`x86_64` or `aarch64`. Run `uname -m` if unsure of your architecture.

**Linux:**
```bash
tar -xzf bmad-method-{version}-linux-{arch}.tar.gz
cp bmad-method-{version}-linux-{arch}/libbmad_plugin.so  ~/.pulse/plugins/bmad-method/
cp bmad-method-{version}-linux-{arch}/plugin.toml        ~/.pulse/plugins/bmad-method/
```

**macOS:**
```bash
tar -xzf bmad-method-{version}-darwin-{arch}.tar.gz
cp bmad-method-{version}-darwin-{arch}/libbmad_plugin.dylib ~/.pulse/plugins/bmad-method/
cp bmad-method-{version}-darwin-{arch}/plugin.toml          ~/.pulse/plugins/bmad-method/
```

> **Required files:** Only the plugin binary (`libbmad_plugin.so` on Linux,
> `libbmad_plugin.dylib` on macOS) and `plugin.toml` are required. `README.md` is included
> in the tarball for convenience but is not needed in the plugin directory.

#### 4. macOS Gatekeeper (macOS only)

If Pulse fails to load the plugin on macOS, the binary may be quarantined by Gatekeeper
because it was downloaded via browser or an untrusted transfer mechanism. Remove the
quarantine attribute before loading:

```bash
xattr -d com.apple.quarantine ~/.pulse/plugins/bmad-method/libbmad_plugin.dylib
```

To check whether the file is quarantined before attempting a load:
```bash
xattr -l ~/.pulse/plugins/bmad-method/libbmad_plugin.dylib
```
No output means the file is not quarantined.

#### 5. File Permissions

The plugin binary is loaded via `dlopen()` — it is **not** executed directly — so it does
**not** require execute (`+x`) permissions. If Pulse reports a permission error, ensure the
file is readable:

```bash
chmod 644 ~/.pulse/plugins/bmad-method/libbmad_plugin.so    # Linux
chmod 644 ~/.pulse/plugins/bmad-method/libbmad_plugin.dylib # macOS
```

#### 6. Load and Verify

```bash
# Reload plugins if Pulse is already running:
pulse plugin reload

# Or restart Pulse to trigger automatic plugin discovery on startup.
```

Confirm the plugin loaded successfully:
```bash
pulse plugin list
# Expected: bmad-method v{version} — 12 agents
# Note: Pulse may display "(manual)" to indicate a manually installed plugin:
#   bmad-method (manual) v{version} — 12 agents
# Both formats are normal.
```

#### Troubleshooting Manual Installation

**`pulse plugin list` does not show `bmad-method`:**
- Confirm the plugin directory contains both required files:
  ```bash
  ls -la ~/.pulse/plugins/bmad-method/
  # Should contain: libbmad_plugin.so (or .dylib) and plugin.toml
  ```
- Check Pulse logs for load errors (typically printed to stderr on startup).
- If the directory exists but is empty or incomplete, re-run the copy commands from Step 3.

**API version mismatch warning:**
```
Warning: bmad-method plugin api_version "1" does not match Pulse api "2"
         The plugin may not work correctly. Consider updating bmad-method.
```
The plugin's `plugin.toml` declares `api_version = 1` (an integer constant tied to the
Pulse plugin API). This warning means your Pulse installation has moved to a newer API.
Download the latest plugin release that matches your Pulse version.

**Wrong platform binary (plugin fails to load):**
Linux and macOS binaries are not interchangeable. Ensure you downloaded the tarball matching
your OS (`linux` vs `darwin`) and CPU architecture (`x86_64` vs `aarch64`).

**macOS: binary quarantined:**
See [Step 4](#4-macos-gatekeeper-macos-only) above. Run `xattr -d com.apple.quarantine` on
the `.dylib` file.

### Source Build (Contributors)

```bash
git clone https://github.com/<YOUR-ORG>/bmad-method
cd bmad-method
cargo run -p bmad-converter -- --input agents/ --output crates/bmad-plugin/src/generated/
cargo build -p bmad-plugin --release
```

Then install the built artifact into Pulse:
```bash
mkdir -p ~/.pulse/plugins/bmad-method
cp target/release/libbmad_plugin.so ~/.pulse/plugins/bmad-method/   # Linux
cp plugin.toml ~/.pulse/plugins/bmad-method/
pulse plugin reload
```

> **macOS:** use `libbmad_plugin.dylib` instead of `.so`.

## Agent Reference

All BMAD agents are available under the `bmad/` namespace. Reference them in workflow YAML
using the exact executor name shown below. Executor names are case-sensitive.

| Agent | Executor Name | Specialization |
|-------|---------------|----------------|
| Winston the Architect | `bmad/architect` | Expert software architect specializing in system design, architectural review, and technical decision-making |
| Amelia the Developer | `bmad/dev` | Ultra-succinct, technically precise developer focused on clean implementation |
| John the PM | `bmad/pm` | Requirements-focused product manager who relentlessly asks WHY and validates user value |
| Quinn the QA Engineer | `bmad/qa` | Practical, test-focused QA engineer with "ship it and iterate" philosophy |
| Maya the UX Designer | `bmad/ux-designer` | Design-focused UX designer specializing in user research, interaction design, and usability analysis |
| Bob the Scrum Master | `bmad/sm` | Agile facilitator specializing in sprint ceremonies, impediment removal, and team coaching |
| Alex the Business Analyst | `bmad/analyst` | Business analyst specializing in requirements elicitation, domain modeling, and process analysis |
| Taylor the Tech Writer | `bmad/tech-writer` | Technical writer specializing in developer documentation, API references, and user guides |
| Barry the Quick Dev | `bmad/quick-flow-solo-dev` | Fast-moving solo developer who creates lean specs and implements small features end-to-end |
| BMad Master | `bmad/bmad-master` | Meta-agent with holistic BMAD-METHOD knowledge for complex multi-domain orchestration |
| Sage the Security Reviewer | `bmad/security` | Security engineer specializing in threat modeling, code review, and security architecture |
| Devon the DevOps Engineer | `bmad/devops` | DevOps engineer specializing in CI/CD pipelines, infrastructure as code, and deployment automation |

> To add a new agent, append one row to this table (see [Adding New Agents](#adding-new-agents)).

---

### bmad/architect — Winston the Architect

**Specialization:** Expert software architect specializing in system design, architectural review, and technical decision-making.

Winston thinks in terms of long-term maintainability and system coherence. With 20+ years of
experience designing distributed systems and APIs, he surfaces risks, trade-offs, and
alternatives before committing to a design.

**Capabilities:** `architecture-review`, `system-design`, `technical-decisions`, `pattern-selection`, `scalability-analysis`, `api-design`, `trade-off-analysis`

**Minimal workflow YAML:**
```yaml
workflow:
  name: architecture-review
  steps:
    - name: review
      executor: bmad/architect
      input: |
        Review this microservice design for scalability:
        {{ context.design_doc }}
```

---

### bmad/dev — Amelia the Developer

**Specialization:** Ultra-succinct, technically precise developer focused on clean implementation.

Amelia communicates with extreme precision and minimal words — dense with technical content,
free of filler. She produces working, tested, production-ready code and values code over
explanation.

**Capabilities:** `code-implementation`, `code-review`, `refactoring`, `debugging`, `technical-documentation`, `test-writing`

**Minimal workflow YAML:**
```yaml
workflow:
  name: implement-feature
  steps:
    - name: implement
      executor: bmad/dev
      input: |
        Implement the feature described in:
        {{ context.spec }}
```

---

### bmad/pm — John the PM

**Specialization:** Requirements-focused product manager who relentlessly asks WHY and validates user value.

John has 15+ years driving product strategy for B2B and developer tools. He is relentlessly
focused on user value and deeply skeptical of features that cannot be tied to a clear user
need or business outcome.

**Capabilities:** `requirements-validation`, `user-story-creation`, `prioritization`, `stakeholder-communication`, `roadmap-planning`, `acceptance-criteria-review`

**Minimal workflow YAML:**
```yaml
workflow:
  name: validate-requirements
  steps:
    - name: validate
      executor: bmad/pm
      input: |
        Validate these requirements against user needs:
        {{ context.requirements }}
```

---

### bmad/qa — Quinn the QA Engineer

**Specialization:** Practical, test-focused QA engineer with "ship it and iterate" philosophy.

Quinn is a pragmatic QA engineer who has shipped hundreds of features across startups and
enterprises. She believes quality is a product property, not a gating function — surface the
highest-risk issues quickly so the team can ship with confidence.

**Capabilities:** `test-planning`, `test-case-generation`, `bug-analysis`, `quality-assessment`, `acceptance-criteria-review`, `risk-assessment`

**Minimal workflow YAML:**
```yaml
workflow:
  name: test-plan
  steps:
    - name: plan
      executor: bmad/qa
      input: |
        Create a test plan for this implementation:
        {{ context.implementation }}
```

---

### bmad/ux-designer — Maya the UX Designer

**Specialization:** Design-focused UX designer specializing in user research, interaction design, and usability analysis.

Maya is a senior UX designer with deep expertise in user research, interaction design, and
design systems. She leads with empathy for users and translates their needs into clear,
actionable design specifications.

**Capabilities:** `ux-research`, `wireframe-design`, `user-journey-mapping`, `usability-analysis`, `interaction-design`, `accessibility-review`

**Minimal workflow YAML:**
```yaml
workflow:
  name: ux-review
  steps:
    - name: review
      executor: bmad/ux-designer
      input: |
        Analyze the user journey and identify friction points in:
        {{ context.user_flow }}
```

---

### bmad/sm — Bob the Scrum Master

**Specialization:** Agile facilitator specializing in sprint ceremonies, impediment removal, and team coaching.

Bob is an experienced Scrum Master and agile coach. A servant leader — his job is to remove
obstacles, facilitate alignment, and protect the team's focus through agile ceremonies and
coaching.

**Capabilities:** `sprint-planning`, `retrospective-facilitation`, `impediment-identification`, `agile-coaching`, `backlog-refinement`, `daily-standup-facilitation`

**Minimal workflow YAML:**
```yaml
workflow:
  name: sprint-planning
  steps:
    - name: plan
      executor: bmad/sm
      input: |
        Facilitate sprint planning for the following backlog:
        {{ context.backlog }}
```

---

### bmad/analyst — Alex the Business Analyst

**Specialization:** Business analyst specializing in requirements elicitation, domain modeling, and process analysis.

Alex excels at translating complex business problems into clear, implementable requirements.
He asks the right questions before proposing solutions and documents requirements with enough
precision that developers can implement without ambiguity.

**Capabilities:** `requirements-elicitation`, `domain-modeling`, `process-analysis`, `stakeholder-mapping`, `gap-analysis`, `use-case-documentation`

**Minimal workflow YAML:**
```yaml
workflow:
  name: requirements-analysis
  steps:
    - name: analyze
      executor: bmad/analyst
      input: |
        Analyze the business requirements for:
        {{ context.problem_statement }}
```

---

### bmad/tech-writer — Taylor the Tech Writer

**Specialization:** Technical writer specializing in developer documentation, API references, and user guides.

Taylor believes documentation is a product feature — if users can't figure out how to use a
feature from the docs, the feature doesn't work. She writes with minimal but complete content
and progressive disclosure.

**Capabilities:** `api-documentation`, `user-guide-creation`, `readme-writing`, `changelog-writing`, `onboarding-documentation`, `content-structure-design`

**Minimal workflow YAML:**
```yaml
workflow:
  name: write-docs
  steps:
    - name: document
      executor: bmad/tech-writer
      input: |
        Write API documentation for:
        {{ context.api_spec }}
```

---

### bmad/quick-flow-solo-dev — Barry the Quick Dev

**Specialization:** Fast-moving solo developer who creates lean specs and implements small features end-to-end.

Barry excels at turning vague requirements into working code with minimal overhead. He creates
lean specs just detailed enough to guide implementation, then builds the solution himself.
No bureaucracy, no over-engineering — just working code, shipped fast.

**Capabilities:** `lean-spec-creation`, `rapid-implementation`, `quick-prototyping`, `small-feature-development`, `fast-iteration`, `minimal-viable-solution`

**Minimal workflow YAML:**
```yaml
workflow:
  name: quick-dev
  steps:
    - name: spec-and-build
      executor: bmad/quick-flow-solo-dev
      input: |
        Quick spec and implement:
        {{ context.requirement }}
```

---

### bmad/bmad-master — BMad Master

**Specialization:** Meta-agent with holistic BMAD-METHOD knowledge for complex multi-domain orchestration.

The BMad Master has comprehensive knowledge of the BMAD-METHOD system and all its specialist
agents. Use this agent when a task spans multiple domains and you need intelligent agent
selection, sequencing, and synthesis across the full roster.

**Capabilities:** `multi-agent-orchestration`, `workflow-design`, `methodology-guidance`, `agent-selection`, `cross-domain-coordination`, `bmad-method-expertise`

**Minimal workflow YAML:**
```yaml
workflow:
  name: orchestrate
  steps:
    - name: orchestrate
      executor: bmad/bmad-master
      input: |
        Orchestrate a full BMAD workflow for:
        {{ context.project_brief }}
```

---

### bmad/security — Sage the Security Reviewer

**Specialization:** Security engineer specializing in threat modeling, code review, and security architecture.

Sage approaches every system as an attacker first and a defender second — understanding how
things can be broken is the only way to build them securely. She conducts threat modeling,
reviews code for vulnerabilities, and defines authentication and authorization requirements.

**Capabilities:** `threat-modeling`, `security-code-review`, `vulnerability-assessment`, `authentication-design`, `authorization-review`, `dependency-auditing`

**Minimal workflow YAML:**
```yaml
workflow:
  name: security-review
  steps:
    - name: review
      executor: bmad/security
      input: |
        Conduct a threat model and security review for:
        {{ context.system_design }}
```

---

### bmad/devops — Devon the DevOps Engineer

**Specialization:** DevOps engineer specializing in CI/CD pipelines, infrastructure as code, and deployment automation.

Devon bridges the gap between development and operations. He believes in automating
everything that can be automated and making deployments a non-event — infrastructure,
pipelines, and config all belong in version control.

**Capabilities:** `cicd-pipeline-design`, `infrastructure-as-code`, `deployment-automation`, `container-orchestration`, `monitoring-setup`, `incident-response`

**Minimal workflow YAML:**
```yaml
workflow:
  name: ci-setup
  steps:
    - name: design-pipeline
      executor: bmad/devops
      input: |
        Design a CI/CD pipeline for:
        {{ context.service_spec }}
```

---

## Workflow Examples

### Example 1: Single-Agent Architecture Review

A simple single-step workflow for architectural feedback.

```yaml
# architecture-review.yaml
workflow:
  name: architecture-review
  steps:
    - name: review
      executor: bmad/architect
      input: |
        Review this API design for scalability and maintainability concerns.
        Focus on: data model choices, service boundaries, and failure modes.

        API spec:
        {{ context.api_spec }}
```

### Example 2: Full Feature Development Pipeline (DAG)

A multi-agent DAG workflow where implementation, testing, and documentation run in parallel
after the design step completes.

```yaml
# feature-development.yaml
workflow:
  name: feature-development
  steps:
    - name: design
      executor: bmad/architect
      input: "Design the data model and API surface for {{ context.feature_name }}"

    - name: implement
      executor: bmad/dev
      depends_on: [design]
      input: |
        Implement the feature based on the architecture:
        {{ steps.design.output }}

    - name: test
      executor: bmad/qa
      depends_on: [implement]
      input: |
        Create a test plan and test cases for the implementation:
        {{ steps.implement.output }}

    - name: document
      executor: bmad/tech-writer
      depends_on: [implement]
      input: |
        Write API documentation for:
        {{ steps.implement.output }}
```

### Example 3: Requirements and Planning Workflow (DAG with parallel branches)

Analyst gathers requirements, then PM and UX Designer work in parallel before the Scrum
Master creates the sprint backlog.

```yaml
# requirements-planning.yaml
workflow:
  name: requirements-planning
  steps:
    - name: analyze
      executor: bmad/analyst
      input: "Analyze the business requirements for {{ context.project_name }}"

    - name: product_plan
      executor: bmad/pm
      depends_on: [analyze]
      input: |
        Create a product plan from these requirements:
        {{ steps.analyze.output }}

    - name: ux_plan
      executor: bmad/ux-designer
      depends_on: [analyze]
      input: |
        Design the UX approach based on:
        {{ steps.analyze.output }}

    - name: sprint_backlog
      executor: bmad/sm
      depends_on: [product_plan, ux_plan]
      input: |
        Create a sprint backlog from:
        Product plan: {{ steps.product_plan.output }}
        UX plan: {{ steps.ux_plan.output }}
```

## Adding New Agents

To add a new BMAD agent to the plugin:

1. Create `agents/{name}.md` with valid frontmatter (see `docs/bmad-frontmatter-schema.md`):
   ```yaml
   ---
   name: agent-name
   displayName: "Display Name"
   description: "One-line description of specialization"
   executor: bmad/agent-name
   capabilities:
     - capability-one
     - capability-two
   ---
   ```
2. Run the converter: `cargo run -p bmad-converter -- --input agents/ --output crates/bmad-plugin/src/generated/`
3. Rebuild the plugin: `cargo build -p bmad-plugin --release`
4. **Update this README:** append one row to the Agent Reference table and add a new `###` subsection below with the agent's details (capabilities list and minimal YAML snippet)

Adding a new agent requires one table row and one detail subsection — a self-contained addition with no changes to existing content.

## Plugin Verification

### Verify Installation Health

```bash
pulse plugin verify bmad-method
# ✓ All 12 agents loaded successfully
```

### View Plugin Metadata

```bash
pulse plugin info bmad-method
# Plugin: bmad-method
# Version: 1.0.0
# API Compatibility: ✓ Compatible
# Agents: 12
# Executors: bmad/architect, bmad/dev, bmad/pm, ...
```

### Troubleshooting

If `pulse plugin verify` fails:
- **"manifest file missing"**: Re-extract the tarball to `~/.pulse/plugins/bmad-method/`
- **"binary not found"**: Check the binary exists and is the correct platform (Linux `.so` vs macOS `.dylib`)
- **"API version mismatch"**: Update the plugin to a version compatible with your Pulse installation

## Plugin Lifecycle Management

All lifecycle operations use standard Pulse CLI commands. These work identically for
CLI-installed and manually-installed plugins.

### Uninstall

```bash
pulse plugin uninstall bmad-method
```

Removes `libbmad_plugin.so` (Linux) / `libbmad_plugin.dylib` (macOS) and `plugin.toml`
from `~/.pulse/plugins/bmad-method/`. After uninstall, `pulse plugin list` no longer shows
`bmad-method` and Pulse will not attempt to load it on next startup.

After uninstalling, any workflow step referencing a `bmad/*` executor will receive a clean
error — not a crash:
```
Error: No executor found for 'bmad/architect'
Hint: The 'bmad-method' plugin may not be installed.
      Run: pulse plugin install bmad-method
```

### Update

```bash
# Update to latest version
pulse plugin update bmad-method

# Update to a specific version (flag availability is Pulse-version-dependent)
pulse plugin update bmad-method --version 1.1.0
```

Pulse compares the `version` field in the installed `plugin.toml` against the latest GitHub
Release tag, downloads the new tarball if a newer version is available, and replaces the
binary and `plugin.toml` atomically.

If the plugin is already at the latest version:
```
bmad-method is already up to date (v1.0.0)
```
The command exits with code 0 and the existing binary is not overwritten.

**Update Safety:** If an update fails mid-download due to a network error, re-run
`pulse plugin update bmad-method` to retry. The Pulse CLI is responsible for atomic
file replacement — if Pulse does not implement atomic updates, both the binary and
`plugin.toml` may be in an inconsistent state after a partial write. In that case,
use the manual rollback steps below to restore a known-good version.

### Rollback (Manual)

If you need to roll back to a previous version (replace `{prev-version}` with the
version you want, e.g. `1.0.0`):
```bash
pulse plugin uninstall bmad-method
pulse plugin install bmad-method --version {prev-version}
```

### Clean Unload (No Global State)

The plugin is safe to unload via `dlclose()`. `registry.rs` uses `std::sync::OnceLock`
to initialize agent metadata — no `lazy_static!` or `once_cell::sync::Lazy` singletons
are used. All static data (agent names, descriptions, prompts) lives in the binary's
data segment and is cleaned up automatically when the binary is unloaded. There is no
heap-allocated global state that leaks on unload.

> All lifecycle management commands work for both CLI-installed and manually-installed plugins.

## Compatibility

| Pulse Version | Status |
|---------------|--------|
| v0.9.x | ✅ Supported |
| v0.8.x | ❌ Not supported |

| Platform | Architecture | Status |
|----------|--------------|--------|
| Linux | x86_64 | ✅ |
| Linux | aarch64 | ✅ |
| macOS | x86_64 | ✅ |
| macOS | aarch64 | ✅ (Apple Silicon) |
| Windows | — | ❌ Not supported |
