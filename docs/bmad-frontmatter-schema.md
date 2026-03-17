# BMAD Agent Frontmatter Schema

**Verified:** 2026-03-17
**Source:** All 12 agent `.md` files in `agents/`
**Story:** 3.1 — BMAD Frontmatter Schema and pulse-api Contract Verification

This document defines the complete YAML frontmatter schema for BMAD agent definition files. It was derived by cataloguing every frontmatter field across all 12 existing agents and confirming parser behaviour in `crates/bmad-converter/src/parser/frontmatter.rs`.

---

## Agents Catalogued

| File | `name` | `executor` | Notes |
|------|--------|------------|-------|
| `agents/analyst.md` | `analyst` | `bmad/analyst` | name == executor suffix |
| `agents/architect.md` | `architect` | `bmad/architect` | name == executor suffix |
| `agents/bmad-master.md` | `bmad-master` | `bmad/bmad-master` | name == executor suffix |
| `agents/developer.md` | `developer` | `bmad/dev` | **name ≠ executor suffix** |
| `agents/devops.md` | `devops` | `bmad/devops` | name == executor suffix |
| `agents/pm.md` | `pm` | `bmad/pm` | name == executor suffix |
| `agents/qa.md` | `qa` | `bmad/qa` | name == executor suffix |
| `agents/quick-flow.md` | `quick-flow` | `bmad/quick-flow` | name == executor suffix |
| `agents/scrum-master.md` | `scrum-master` | `bmad/sm` | **name ≠ executor suffix** |
| `agents/security.md` | `security` | `bmad/security` | name == executor suffix |
| `agents/tech-writer.md` | `tech-writer` | `bmad/tech-writer` | name == executor suffix |
| `agents/ux-designer.md` | `ux-designer` | `bmad/ux-designer` | name == executor suffix |

**Important:** `executor` does NOT have to equal `"bmad/" + name`. `developer.md` uses `bmad/dev` and `scrum-master.md` uses `bmad/sm`. The invariant is that `executor` must start with the `bmad/` namespace prefix.

---

## Required Fields

All 5 fields below are present in every agent file and must be present for the converter to succeed. Missing any one of them causes `parse_file()` to return `Err`.

| Field | YAML Type | Description | Example |
|-------|-----------|-------------|---------|
| `name` | string | Machine identifier — unique, lowercase, hyphens allowed | `architect` |
| `displayName` | string | Human-readable display name for UI | `"Winston the Architect"` |
| `description` | string | One-line capability summary shown in agent listings | `"Expert software architect specializing in system design"` |
| `executor` | string | Pulse executor path — must start with `bmad/` | `bmad/architect` |
| `capabilities` | list of strings | Agent domain competencies (minimum 1 entry required) | `- architecture-review` |

---

## Optional Fields

| Field | YAML Type | Default | Description | Example |
|-------|-----------|---------|-------------|---------|
| `temperature` | float (f32) | `None` | Suggested LLM sampling temperature (0.0–2.0). When absent, `ParsedAgent.temperature` is `None` and the caller uses its own default. | `0.7` |

No other optional fields were found across the 12 catalogued agents.

---

## Validation Rules

1. **`name`** — must be lowercase; only `[a-z0-9-]` characters are conventional (hyphens allowed, no spaces). The parser does not enforce this constraint at parse time but the code generator relies on it for module naming.

2. **`executor`** — must start with `"bmad/"` by convention. The suffix after `bmad/` is the executor identifier registered in the Pulse host. It does **not** have to equal `name` (see `developer` → `bmad/dev`, `scrum-master` → `bmad/sm`). **Note: the parser does NOT enforce the `bmad/` prefix at parse time** — an agent with `executor: other/executor` will parse successfully. The prefix is a documented convention enforced by the Pulse host at plugin load time, not by the converter. This gap was identified in Story 3.1 and was not added to Story 1.3 (already done) — validation belongs in a future enhancement.

3. **`capabilities`** — must be a YAML list using `- item` syntax, not a comma-separated string. The parser deserializes it as `Vec<String>`. Must have at least 1 entry (the parser does not enforce a minimum but an empty list would produce an agent with no capabilities).

4. **All required fields must be present** — `parse_file()` returns `Err` with a descriptive message naming the missing field and the file path. Error format: `"missing required field '{field}' in {path}"`.

5. **`displayName` is camelCase in YAML** — the Rust field is `display_name` (snake_case) but the YAML key is `displayName`. The parser uses `#[serde(rename = "displayName")]`.

---

## Complete Example

A full valid frontmatter block for a new hypothetical agent (`my-specialist`):

```yaml
---
name: my-specialist
displayName: "Maya the Specialist"
description: "Provides expert guidance on specialized domain tasks"
executor: bmad/my-specialist
capabilities:
  - domain-analysis
  - task-decomposition
  - recommendation-generation
---
```

> Only one capability entry is required; three are shown above for illustration.

With optional temperature field:

```yaml
---
name: my-specialist
displayName: "Maya the Specialist"
description: "Provides expert guidance on specialized domain tasks"
executor: bmad/my-specialist
capabilities:
  - domain-analysis
  - task-decomposition
  - recommendation-generation
temperature: 0.3
---
```

---

## Mapping to Rust Types

### `ParsedAgent` (converter output)

After `parse_file()` succeeds, the result is a `ParsedAgent`:

| Frontmatter Field | `ParsedAgent` Field | Rust Type |
|-------------------|---------------------|-----------|
| `name` | `name` | `String` |
| `displayName` | `display_name` | `String` |
| `description` | `description` | `String` |
| `executor` | `executor_name` | `String` |
| `capabilities` | `capabilities` | `Vec<String>` |
| `temperature` | `temperature` | `Option<f32>` |
| (file body after `---`) | `body` | `String` |

### `AgentMetadata` (generated static type)

After code generation, each agent is compiled into a static `AgentMetadata`:

| Frontmatter Field | `AgentMetadata` Field | Rust Type |
|-------------------|-----------------------|-----------|
| `name` | `name` and `id` | `&'static str` |
| `displayName` | `display_name` | `&'static str` |
| `description` | `description` | `&'static str` |
| `executor` | `executor_name` | `&'static str` |
| `capabilities` | `capabilities` | `&'static [&'static str]` |

Note: `AgentMetadata.id` is set to the same value as `name`. There is no separate `id` field in the frontmatter — `id` is always derived from `name` during code generation.

---

## Parser Error Messages

The converter returns descriptive errors that follow the pattern:

```
missing required field '{field}' in {path}
```

Examples:
- `missing required field 'name' in agents/my-agent.md`
- `missing required field 'displayName' in agents/my-agent.md`
- `missing required field 'executor' in agents/my-agent.md`
- `failed to parse frontmatter in agents/my-agent.md: ...`
- `file is empty: agents/my-agent.md`
