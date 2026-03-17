# Story 3.3: Write Comprehensive Agent Reference Documentation

Status: in-progress

## Story

As a Pulse workflow author,
I want comprehensive documentation that lists all BMAD agents, their capabilities, executor names, and workflow YAML examples,
so that I can discover the right agent for my task without reading source code or trial and error.

## Acceptance Criteria

**AC1 — README.md contains installation, agent table, and YAML examples:**

**Given** the `README.md` exists in the repository root
**When** a user reads it
**Then** it contains all of: installation instructions (both CLI and manual methods), a complete agent reference table with all 12+ agents showing executor name and one-line description, and at least 2 complete workflow YAML examples

**AC2 — Agent table matches registry exactly (no phantoms, no gaps):**

**Given** the agent reference table in `README.md`
**When** compared against the registered agents in the plugin
**Then** every registered `bmad/` executor has a corresponding row in the table
**And** every row in the table corresponds to an actual registered executor (no phantom entries)

**AC3 — Each agent entry includes specialization, executor name, and YAML snippet:**

**Given** a workflow author wants to use `bmad/architect`
**When** they find it in the README
**Then** they can read: what the Architect agent specializes in, the exact executor name to use, and a minimal workflow YAML snippet showing it in a step definition

**AC4 — Adding a new agent requires only one table row:**

**Given** a new agent `.md` file is added to `agents/`
**When** the README agent table is updated
**Then** the update requires only appending one row — no structural changes to the README are needed

**AC5 — All documented executor names match registry exactly:**

**Given** the README contains executor names
**When** every executor name in the documentation is compared against `registry.list_agents()`
**Then** all documented executor names match actual registered names exactly (no typos or outdated names)

## Tasks / Subtasks

- [x] **Task 1: Gather ground-truth agent list from registry** (AC: #2, #5)
  - [x] Run `registry.list_agents()` output (or inspect `agents/` directory) to get the definitive list of executor names and descriptions
  - [x] For each agent, extract from its `.md` file:
    - `executor` field (exact executor name, e.g., `bmad/architect`)
    - `displayName` field (human-readable name, e.g., `"Winston the Architect"`)
    - `description` field (one-line summary)
    - `capabilities` list (for the per-agent detail section)
  - [x] Compile into a master list (working notes) before writing anything to README.md
  - [x] Verify no two agents share the same executor name
  - [x] Confirm all executor names follow `bmad/{identifier}` with only `[a-z0-9-]` in identifier

- [x] **Task 2: Draft README.md structure** (AC: #1, #3, #4)
  - [x] Plan the README sections (do not write yet — plan first):
    1. Title + badge line
    2. One-paragraph overview ("BMAD-METHOD plugin for Pulse — 12+ specialized AI agents")
    3. Quick Start (install command + immediate first workflow)
    4. Installation (CLI + Manual + Source Build)
    5. Agent Reference (the complete table)
    6. Workflow Examples (≥2 complete YAML examples)
    7. Adding New Agents (developer section)
    8. Compatibility
  - [x] Agent Reference table column design: `| Agent | Executor Name | Specialization |`
    - Each row: one agent, one executor name, one specialization sentence
    - The table must be append-only (AC4): adding an agent = adding one `| ... | ... | ... |` row

- [x] **Task 3: Write the README.md** (AC: #1, #2, #3, #4, #5)
  - [x] Open `README.md` at the workspace root (create if it does not exist)
  - [x] Write **Section: BMAD-METHOD Pulse Plugin** (title and overview)
    ```
    # BMAD-METHOD Pulse Plugin

    A Pulse plugin delivering 12+ battle-tested BMAD-METHOD AI agents for workflow orchestration.
    Install once and get a full team of AI specialists — Architect, Developer, PM, QA, UX, and more.
    ```
  - [x] Write **Section: Quick Start**
    ```markdown
    ## Quick Start

    ```bash
    pulse plugin install bmad-method
    ```

    Then reference agents in your workflow:

    ```yaml
    steps:
      - name: review
        executor: bmad/architect
        input: "Review this API design for scalability concerns"
    ```
    ```

  - [x] Write **Section: Installation** with both methods (AC: #1)
    ```markdown
    ## Installation

    ### CLI Installation (Recommended)

    ```bash
    pulse plugin install bmad-method
    ```

    Installs the latest version. After installation, all `bmad/` executors are available in Pulse workflows.

    ### Manual Installation

    1. Download the release archive for your platform from the [GitHub Releases](https://github.com/your-org/bmad-pulse-plugin/releases) page:
       - Linux x86_64: `bmad-method-{version}-linux-x86_64.tar.gz`
       - Linux aarch64: `bmad-method-{version}-linux-aarch64.tar.gz`
       - macOS x86_64: `bmad-method-{version}-darwin-x86_64.tar.gz`
       - macOS aarch64: `bmad-method-{version}-darwin-aarch64.tar.gz`

    2. Extract to the Pulse plugin directory:
       ```bash
       mkdir -p ~/.pulse/plugins/bmad-method
       tar -xzf bmad-method-{version}-{platform}.tar.gz -C ~/.pulse/plugins/bmad-method/
       ```

    3. Reload Pulse plugins:
       ```bash
       pulse plugin reload
       # or restart Pulse
       ```

    4. Verify installation:
       ```bash
       pulse plugin list
       # bmad-method vX.Y.Z — 12 agents
       ```

    > **Note:** Manual installation works in air-gapped environments. The only files required in `~/.pulse/plugins/bmad-method/` are the plugin binary and `plugin.toml`.

    ### Source Build (Contributors)

    ```bash
    git clone https://github.com/your-org/bmad-pulse-plugin
    cd bmad-pulse-plugin
    cargo run -p bmad-converter -- --input agents/ --output crates/bmad-plugin/src/generated/
    cargo build -p bmad-plugin --release
    # Binary: target/release/libbmad_plugin.{so,dylib}
    ```
    ```

  - [x] Write **Section: Agent Reference** — the complete table (AC: #2, #3, #4, #5)

    The table must list ALL registered agents. Use the ground-truth list from Task 1. Below is the template — populate with actual data from the agent files:

    ```markdown
    ## Agent Reference

    All BMAD agents are available under the `bmad/` namespace. Reference them in workflow YAML using the exact executor name shown below.

    | Agent | Executor Name | Specialization |
    |-------|---------------|----------------|
    | Winston the Architect | `bmad/architect` | Architecture review, system design, technical decision-making |
    | Amelia the Developer | `bmad/dev` | Code implementation, debugging, technical problem-solving |
    | John the PM | `bmad/pm` | Product requirements, user value validation, WHY-focused questioning |
    | Quinn the QA | `bmad/qa` | Test strategy, bug analysis, quality assurance, "ship it and iterate" |
    | Aria the UX Designer | `bmad/ux-designer` | UX patterns, user empathy, interface design guidance |
    | Bob the Scrum Master | `bmad/sm` | Agile ceremonies, sprint planning, team facilitation |
    | Alex the Analyst | `bmad/analyst` | Business analysis, requirements elicitation, domain research |
    | Taylor the Tech Writer | `bmad/tech-writer` | Technical documentation, API docs, user guides |
    | Riley the Quick Flow | `bmad/quick-flow` | Rapid prototyping, quick specs, fast-turnaround deliverables |
    | BMAD Master | `bmad/bmad-master` | Workflow orchestration, multi-agent coordination, BMAD methodology |
    | *(add rows here for additional agents)* | | |

    > The executor name column shows the exact string to use in `executor:` fields. Names are case-sensitive.
    ```

    **CRITICAL:** Before writing this table, verify each row against the actual agent `.md` files:
    - `displayName` → Agent column
    - `executor` → Executor Name column (copy verbatim — no paraphrasing)
    - `description` → Specialization column (use description field, may extend slightly for clarity)
    - If an agent's actual executor name differs from what's listed above, use the actual value

  - [x] Write per-agent detail subsections (AC: #3) — one H3 per agent:
    ```markdown
    ### bmad/architect — Winston the Architect

    **Specialization:** Architecture review, system design decisions, technical standards.

    Winston thinks in terms of long-term maintainability and system coherence. Use this agent
    when you need opinionated architectural guidance grounded in real engineering principles.

    **Capabilities:** architecture-review, system-design, api-design, technical-documentation, adr-creation

    **Minimal workflow YAML:**
    ```yaml
    steps:
      - name: architecture-review
        executor: bmad/architect
        input: |
          Review this microservice design for scalability:
          {{ context.design_doc }}
    ```
    ```

    Repeat for each agent. Each subsection must include: specialization sentence, 2-3 sentence description of the agent's personality/approach, capabilities list (from frontmatter), and a minimal workflow YAML snippet.

  - [x] Write **Section: Workflow Examples** with ≥2 complete examples (AC: #1)

    **Example 1: Architecture Review Workflow**
    ```markdown
    ## Workflow Examples

    ### Example 1: Architecture Review

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

    ### Example 2: Full Development Pipeline

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
            Create a test plan for the implementation:
            {{ steps.implement.output }}

        - name: document
          executor: bmad/tech-writer
          depends_on: [implement]
          input: |
            Write API documentation for:
            {{ steps.implement.output }}
    ```

    ### Example 3: Requirements and Planning Workflow

    ```yaml
    # requirements-planning.yaml
    workflow:
      name: requirements-planning
      steps:
        - name: analyze
          executor: bmad/analyst
          input: "Analyze the business requirements for {{ context.project_name }}"

        - name: product-plan
          executor: bmad/pm
          depends_on: [analyze]
          input: |
            Create a product plan from these requirements:
            {{ steps.analyze.output }}

        - name: ux-plan
          executor: bmad/ux-designer
          depends_on: [analyze]
          input: |
            Design the UX approach for:
            {{ steps.analyze.output }}
    ```
    ```

  - [x] Write **Section: Adding New Agents** (developer reference) (AC: #4)
    ```markdown
    ## Adding New Agents

    To add a new BMAD agent to the plugin:

    1. Create `agents/{name}.md` with valid frontmatter (see `docs/bmad-frontmatter-schema.md`)
    2. Run the converter: `cargo run -p bmad-converter -- --input agents/ --output crates/bmad-plugin/src/generated/`
    3. Rebuild the plugin: `cargo build -p bmad-plugin --release`
    4. **Update this README:** append one row to the Agent Reference table above

    The table is designed for append-only updates — no structural changes needed.
    ```

  - [x] Write **Section: Compatibility**
    ```markdown
    ## Compatibility

    | Pulse Version | Status |
    |---------------|--------|
    | v0.9.x | ✅ Supported |
    | v0.8.x | ❌ Not supported |

    | Platform | Architecture | Status |
    |----------|-------------|--------|
    | Linux | x86_64 | ✅ |
    | Linux | aarch64 | ✅ |
    | macOS | x86_64 | ✅ |
    | macOS | aarch64 | ✅ (Apple Silicon) |
    | Windows | — | ❌ Not supported |
    ```

- [x] **Task 4: Consistency verification pass** (AC: #2, #5)
  - [x] For every executor name that appears in `README.md`, compare it character-by-character against the corresponding entry in `registry.list_agents()` (or the `executor` field in the agent's `.md` file)
  - [x] Fix any typos, outdated names, or casing differences
  - [x] Verify every registered agent has a row in the table — if a registered agent is missing, add it
  - [x] Verify every row in the table maps to an actual registered agent — if a row has no corresponding `.md` file + registered executor, remove it
  - [x] Create a simple verification checklist in `docs/README-consistency-check.md` (optional but recommended):
    ```markdown
    # README Consistency Checklist

    Run after any agent add/remove to verify README stays in sync.

    ## Agents in registry vs README table
    - [ ] bmad/analyst ↔ row in README table
    - [ ] bmad/architect ↔ row in README table
    - [ ] bmad/bmad-master ↔ row in README table
    - [ ] bmad/dev ↔ row in README table
    - [ ] bmad/pm ↔ row in README table
    - [ ] bmad/qa ↔ row in README table
    - [ ] bmad/quick-flow ↔ row in README table
    - [ ] bmad/sm ↔ row in README table
    - [ ] bmad/tech-writer ↔ row in README table
    - [ ] bmad/ux-designer ↔ row in README table
    ```

### Review Follow-ups (AI)

- [x] **[AI-Review][HIGH] H1 — Remove hardcoded "12" from overview paragraph** (`README.md` line 3)
- [x] **[AI-Review][HIGH] H2 — Rewrite "Adding New Agents" step 4 and trailing note for consistency** (`README.md` Adding New Agents section)
- [x] **[AI-Review][MEDIUM] M1 — Add post-build install step to Source Build section** (`README.md` Source Build section)
- [x] **[AI-Review][MEDIUM] M2 — Replace `your-org` placeholder with visually distinct `<YOUR-ORG>`** (`README.md` URLs)
- [x] **[AI-Review][LOW] L1 — Rename hyphenated step names to underscores in Example 3** (`README.md` Example 3)

- [x] **Task 5: Final review and validation** (AC: all)
  - [x] Read the complete README.md from top to bottom as a new user would
  - [x] Verify: can a user who knows nothing about BMAD-METHOD install the plugin and write a workflow using only the README? If not, add missing information
  - [x] Verify: all code blocks are syntactically valid YAML (indentation, key names)
  - [x] Verify: all executor names in YAML examples match the Agent Reference table
  - [x] Verify: manual installation steps are accurate and complete (correct directory path, correct file names)
  - [x] Run `cargo test -p bmad-plugin` to confirm registry is still passing (Story 3.2 tests should still pass — this story does not change Rust code)

## Dev Notes

### Documentation-Only Story

This story produces **only documentation**:
- `README.md` at the workspace root (primary deliverable)
- Optional: `docs/README-consistency-check.md`

No Rust code changes are made. No new crates. No Cargo.toml modifications. If a code change is discovered to be necessary, it belongs in Story 3.2 (registry) or earlier stories — do not add code changes here.

### Source of Truth for Agent Data

**Use actual `.md` files as the source of truth**, not the task description's assumptions. The task description provides a plausible list of agents and executor names, but the actual values in `agents/*.md` frontmatter fields are authoritative.

For each agent, always read:
1. `executor` frontmatter field → use verbatim as the Executor Name
2. `displayName` frontmatter field → use as the Agent column value
3. `description` frontmatter field → use as the base for Specialization (may expand slightly)
4. `capabilities` frontmatter field → use verbatim in per-agent detail section

**Known executor names from architecture.md and epics.md** (verify against actual files):
- `bmad/architect` — Architect
- `bmad/dev` — Developer (note: short form "dev", not "developer")
- `bmad/pm` — Product Manager
- `bmad/qa` — QA Engineer
- `bmad/ux-designer` — UX Designer (hyphenated)
- `bmad/sm` — Scrum Master (short form)
- `bmad/analyst` — Business Analyst
- `bmad/tech-writer` — Tech Writer (hyphenated)
- `bmad/quick-flow` — Quick Flow (hyphenated)
- `bmad/bmad-master` — BMAD Master (hyphenated)

### AC2 Anti-Pattern Warning: Phantom Entries

A "phantom entry" is a README table row whose executor name does NOT exist in `registry.list_agents()`. This can happen when:
- An agent was documented but its `.md` file was never created
- An agent was renamed (e.g., `bmad/developer` renamed to `bmad/dev`) but the old name remained in README
- A planned agent was listed speculatively

The AC specifically forbids phantom entries. Remove any row that does not have a corresponding registered executor.

### Workflow YAML Format

The YAML examples must use the actual Pulse workflow format. Based on PRD examples (`prd.md` lines 149–168, 306–328):

```yaml
workflow:
  name: my-workflow
  steps:
    - name: step-name
      executor: bmad/agent-name
      input: "The task text here"
      depends_on: [previous-step-name]  # optional
```

Key format rules:
- `executor:` value is the exact executor name string (e.g., `bmad/architect`)
- `input:` is a string — can be a multi-line literal block with `|`
- `depends_on:` is a YAML list of step names — use to create DAG dependencies
- Template variable syntax: `{{ context.variable_name }}` and `{{ steps.step_name.output }}`

Verify these assumptions against the Pulse workflow documentation at `/home/jack/Document/pulse/docs/`.

### README Maintenance Model

The README is designed for append-only updates to the agent table (AC4). This means:
- The table must NOT have columns that would require structural changes when adding a new agent
- Avoid: a "Count" column that needs updating, or a summary sentence like "Below are the 10 agents" (use "all available agents" instead)
- The per-agent detail subsections (H3 headers) do require adding a new subsection per agent — this is acceptable as it is a single coherent addition

### Story Dependencies

- **Depends on:** Story 3.2 (registry provides the authoritative executor list)
- **Depends on:** Stories 2.3 + 2.4 (agent `.md` files provide displayName, description, capabilities)
- **Parallel safe with:** Story 3.1 (schema docs do not conflict)
- **Blocks:** Nothing — final consumer-facing story of Epic 3

### Avoiding Common README Mistakes

1. **Wrong executor name format:** Use `bmad/dev` not `bmad/developer`. Copy from the `executor` frontmatter field.
2. **Missing YAML indentation:** Pulse YAML is 2-space indented. Do not use tabs.
3. **Outdated installation path:** Verify `~/.pulse/plugins/` is the correct path by checking `/home/jack/Document/pulse/docs/plugin-development-guide.md`
4. **Inconsistent agent descriptions:** The table's "Specialization" column and the per-agent H3 section must describe the same agent consistently — do not contradict each other
5. **Missing manual install steps:** AC1 explicitly requires both CLI and manual installation — do not omit either

### Project Structure Notes

```
(workspace root)/
├── README.md                          ← PRIMARY DELIVERABLE (create or overwrite)
└── docs/
    └── README-consistency-check.md    ← OPTIONAL secondary deliverable
```

The `README.md` sits at the workspace root alongside `Cargo.toml`, `LICENSE`, and `rust-toolchain.toml`. This is standard for Rust projects and is the first file GitHub/GitLab renders for the repository.

### References

- `epics.md` lines 600–630: Story 3.3 full spec with all ACs
- `epics.md` lines 466–494: Story 2.4 — full list of expected agent `.md` files
- `epics.md` lines 429–456: Story 2.3 — Architect, Dev, PM, QA agent file requirements
- `architecture.md` lines 339–417: Full project directory structure (README.md at root)
- `architecture.md` lines 259–270: Naming patterns — executor name conventions
- `prd.md` lines 113–168: User journeys 1 and 2 — how users discover and use agents (guides tone of README)
- `prd.md` lines 117–140: Exact CLI install command and output format example
- `prd.md` lines 149–168: DAG workflow YAML format (multi-agent example to use)
- `prd.md` lines 283–329: Installation methods table and YAML code examples
- `prd.md` lines 344–347: Agent metadata fields to extract from `.md` files

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6 (anthropic/claude-sonnet-4-6)

### Debug Log References

No debug issues. Documentation-only story with no code changes.

### Completion Notes List

- ✅ Read all 12 agent `.md` files to extract exact frontmatter values (displayName, executor, description, capabilities)
- ✅ Compiled authoritative agent roster: all 12 agents present with unique `bmad/` prefixed executor names
- ✅ Verified no duplicate executor names across all agents
- ✅ Rewrote README.md with: Quick Start, CLI + Manual + Source installation, complete 12-agent reference table, per-agent H3 detail sections with capabilities and YAML snippets, 3 complete workflow examples (1 simple + 2 DAG with depends_on), Adding New Agents guide, Compatibility table
- ✅ All 12 executor names verified character-by-character against `agents/*.md` frontmatter `executor` fields
- ✅ Story template had only 10 agents with incorrect displayNames (e.g. "Aria the UX Designer", "Bob the Scrum Master") — corrected to actual values from files ("Maya the UX Designer", "Sam the Scrum Master") and added missing `bmad/security` and `bmad/devops` rows
- ✅ Created `docs/README-consistency-check.md` with all 12 agents listed for ongoing verification
- ✅ Ran `cargo test -p bmad-plugin`: 39 tests pass, no regressions (no Rust code changed)
- ✅ Plugin directory path `~/.pulse/plugins/bmad-method/` confirmed from story 4-3 and 4-6 artifacts
- ✅ Resolved review finding [HIGH] H1: removed hardcoded "12" count → "a full team of battle-tested BMAD-METHOD AI agents"
- ✅ Resolved review finding [HIGH] H2: rewrote Adding New Agents step 4 to honestly state both table row + H3 subsection required; replaced contradictory trailing note with accurate "self-contained addition" wording
- ✅ Resolved review finding [MEDIUM] M1: added post-build install step to Source Build section (copy .so/.dylib + plugin.toml, reload)
- ✅ Resolved review finding [MEDIUM] M2: replaced `your-org` with `<YOUR-ORG>` in both Manual Installation and Source Build URLs; added callout note
- ✅ Resolved review finding [LOW] L1: renamed Example 3 step names from hyphenated (`product-plan`, `ux-plan`) to underscored (`product_plan`, `ux_plan`) and updated all template references accordingly

### File List

- `README.md` (modified — complete rewrite with user-facing documentation)
- `docs/README-consistency-check.md` (created — optional consistency verification checklist)

### Change Log

- 2026-03-17: Rewrote README.md with comprehensive user-facing documentation (all 12 agents, 3 workflow examples, CLI + manual installation). Created docs/README-consistency-check.md. No Rust code changes.
- 2026-03-17: Addressed code review findings — 5 items resolved (2 High, 2 Medium, 1 Low). README.md fixes: removed hardcoded agent count, corrected Adding New Agents wording, added source build install step, replaced ambiguous URL placeholders, renamed hyphenated step names to underscores in Example 3.

---

## Senior Developer Review (AI)

**Reviewer:** claude-sonnet-4-6 (adversarial code review)
**Review Date:** 2026-03-17
**Story:** 3-3-agent-reference-documentation
**Git vs Story Discrepancies:** 0 (README.md and docs/ are untracked new files — consistent with File List)
**Issues Found:** 2 High, 2 Medium, 1 Low

---

### Cross-Reference Verification Results

All 12 executor names in `README.md` were verified character-by-character against `agents/*.md` frontmatter `executor` fields:

| Agent File | Executor (frontmatter) | README Table | Match |
|------------|----------------------|--------------|-------|
| analyst.md | `bmad/analyst` | `bmad/analyst` | ✅ |
| architect.md | `bmad/architect` | `bmad/architect` | ✅ |
| bmad-master.md | `bmad/bmad-master` | `bmad/bmad-master` | ✅ |
| developer.md | `bmad/dev` | `bmad/dev` | ✅ |
| devops.md | `bmad/devops` | `bmad/devops` | ✅ |
| pm.md | `bmad/pm` | `bmad/pm` | ✅ |
| qa.md | `bmad/qa` | `bmad/qa` | ✅ |
| quick-flow.md | `bmad/quick-flow` | `bmad/quick-flow` | ✅ |
| scrum-master.md | `bmad/sm` | `bmad/sm` | ✅ |
| security.md | `bmad/security` | `bmad/security` | ✅ |
| tech-writer.md | `bmad/tech-writer` | `bmad/tech-writer` | ✅ |
| ux-designer.md | `bmad/ux-designer` | `bmad/ux-designer` | ✅ |

No phantom entries, no gaps, no typos. AC2 and AC5 pass.

All per-agent H3 sections verified against frontmatter capabilities lists — all match exactly.

---

### 🔴 HIGH Issues (Must Fix)

#### ✅ H1 — Hardcoded "12" in opening overview paragraph violates AC4 principle [RESOLVED]
**File:** `README.md`, first paragraph (line ~4)
**Text:** `"A Pulse plugin delivering 12 battle-tested BMAD-METHOD AI agents for workflow orchestration."`

**Problem:** The hardcoded count `12` must be updated every time an agent is added or removed. This is precisely the kind of maintenance burden the Dev Notes explicitly warn against:

> *"Avoid: a 'Count' column that needs updating, or a summary sentence like 'Below are the 10 agents' (use 'all available agents' instead)"*

AC4 requires that adding a new agent requires "only appending one row" to the table. A hardcoded count in the paragraph above the table is an additional edit required on every agent addition — violating that principle in spirit.

**Fix:** Replace `"12 battle-tested"` with `"battle-tested"` (drop the count entirely), or use phrasing like `"a full team of battle-tested BMAD-METHOD AI agents"`.

---

#### ✅ H2 — "Adding New Agents" step 4 contradicts AC4 and is internally inconsistent [RESOLVED]
**File:** `README.md`, "Adding New Agents" section, step 4
**Text:** `"**Update this README:** append one row to the Agent Reference table above and add a new \`###\` subsection"`

**Problem (Part A — AC4 violation):**
AC4 states: *"the update requires only appending one row — no structural changes to the README are needed."*
Adding a new `###` H3 subsection IS a structural addition to the document. Step 4 requires two distinct README changes, directly contradicting AC4's "only one row" guarantee.

**Problem (Part B — internal inconsistency):**
The sentence immediately following step 4 says:
> *"The table is designed for append-only updates — no structural changes to the README are needed."*

This sentence contradicts the instruction in the same step that requires adding a new H3 section. A reader following step 4 will be confused: do they need to add a subsection or not?

The Dev Notes acknowledge this (see "README Maintenance Model" note: "this is acceptable as it is a single coherent addition"), but the acceptance in dev notes does not make the published README instruction consistent with AC4 or with itself.

**Fix Options:**
1. Remove the H3 per-agent detail sections entirely and move per-agent capabilities/YAML into the table (e.g., expandable detail or linked anchors). Table-only design would make AC4 strictly true.
2. Keep H3 sections but rewrite step 4 to say: *"Append one row to the Agent Reference table AND add one new `###` subsection"* — and update AC4 or the trailing note to acknowledge two additions are required.
3. At minimum: rewrite the trailing sentence to say *"The table is designed for append-only row additions. Per-agent detail sections also require one new `###` subsection."* — to eliminate the internal contradiction even if AC4 remains technically unmet.

---

### 🟡 MEDIUM Issues (Should Fix)

#### ✅ M1 — Source Build section missing post-build installation step [RESOLVED]
**File:** `README.md`, "Source Build (Contributors)" section

**Problem:** The section ends at `cargo build -p bmad-plugin --release` with no instruction on how to install the built artifact into Pulse. A contributor who builds from source is left stranded:
- The Manual Installation section documents placing files at `~/.pulse/plugins/bmad-method/`
- Source Build produces `target/release/libbmad_plugin.{so,dylib}` but doesn't say which files to copy where
- The Manual Installation note says: *"The only files required in `~/.pulse/plugins/bmad-method/` are the plugin binary and `plugin.toml`"* — but Source Build contributors don't know where `plugin.toml` comes from

**Fix:** Add a final step to Source Build:
```
5. Install into Pulse:
   ```bash
   mkdir -p ~/.pulse/plugins/bmad-method
   cp target/release/libbmad_plugin.{so,dylib} ~/.pulse/plugins/bmad-method/
   cp plugin.toml ~/.pulse/plugins/bmad-method/
   pulse plugin reload
   ```
```

---

#### ✅ M2 — Placeholder GitHub URLs are not visually identifiable as placeholders [RESOLVED]
**File:** `README.md`
- Manual Installation step 1: `https://github.com/your-org/bmad-pulse-plugin/releases`
- Source Build: `git clone https://github.com/your-org/bmad-pulse-plugin`

**Problem:** The placeholder `your-org` uses lowercase without any visual signifier (no angle brackets `<>`, no uppercase `YOUR_ORG`, no inline callout). A user who copy-pastes either line gets a 404 with no explanation. The `git clone` line is especially dangerous as it will silently clone an empty repo if the org happens to exist.

Common placeholder conventions that clearly signal "replace this":
- `<your-org>` (angle brackets)
- `YOUR_ORG` (all-caps)
- `{your-org}` (curly braces)
- Accompanied by a `> **Replace** \`your-org\` with your actual GitHub org` callout

**Fix:** Either replace with the real GitHub URL (if known at this stage), or change to `YOUR_ORG` (all-caps) and add a callout note explaining the substitution.

---

### 🟢 LOW Issues (Nice to Fix)

#### ✅ L1 — Template variable syntax for hyphenated step names unverified against Pulse engine [RESOLVED]
**File:** `README.md`, Example 3 (`requirements-planning.yaml`)
**Lines:**
```yaml
        Product plan: {{ steps.product-plan.output }}
        UX plan: {{ steps.ux-plan.output }}
```

**Problem:** Step names `product-plan` and `ux-plan` contain hyphens. The template reference `{{ steps.product-plan.output }}` uses dot-notation over a hyphenated key. In many common template engines (Jinja2, Handlebars, Liquid), a hyphen in a property name accessed via dot notation is either a parse error or ambiguous (interpreted as subtraction). Whether Pulse's template engine supports this syntax was not verified.

The story Dev Notes explicitly say: *"Verify these assumptions against the Pulse workflow documentation at `/home/jack/Document/pulse/docs/`."* The completion notes do not mention verifying hyphenated step name template syntax.

**Risk:** Low — Pulse may well support this syntax; the issue is that it was not verified.

**Fix:** Check Pulse workflow docs to confirm `{{ steps.hyphenated-name.output }}` is valid. If not, rename steps to use underscores (`product_plan`, `ux_plan`) or camelCase (`productPlan`, `uxPlan`). Either update both the `name:` field and the template reference, and confirm in completion notes.

---

### AC Compliance Summary

| AC | Status | Notes |
|----|--------|-------|
| AC1 — Installation + table + YAML examples | ✅ PASS | CLI, manual, source build all present; 3 YAML examples |
| AC2 — No phantoms, no gaps | ✅ PASS | All 12 executors verified |
| AC3 — Specialization + executor + YAML per agent | ✅ PASS | All 12 H3 sections complete |
| AC4 — Adding agent = one row append | ❌ FAIL | H1 (hardcoded count) + H2 (H3 subsection required) |
| AC5 — Executor names exact | ✅ PASS | All 12 verified character-by-character |

### Recommendation

Address H1 (trivial — remove "12") and H2 (rewrite the "Adding New Agents" step 4 wording and trailing note for consistency). These are small wording fixes, not structural rewrites. M1 and M2 are also straightforward additions.
