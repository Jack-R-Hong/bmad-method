use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct FrontmatterData {
    pub name: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub executor: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ParsedAgent {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub executor_name: String,
    pub capabilities: Vec<String>,
    pub body: String,
    pub temperature: Option<f32>,
}

pub fn parse_file(path: &Path) -> Result<ParsedAgent> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read file: {}", path.display()))?;

    if content.trim().is_empty() {
        return Err(anyhow::anyhow!("file is empty: {}", path.display()));
    }

    let document = yaml_front_matter::YamlFrontMatter::parse::<FrontmatterData>(&content)
        .map_err(|e| anyhow::anyhow!("failed to parse frontmatter in {}: {}", path.display(), e))?;

    let fm = document.metadata;
    let body = document.content.trim().to_string();

    let name = fm
        .name
        .ok_or_else(|| anyhow::anyhow!("missing required field 'name' in {}", path.display()))?;

    let display_name = fm.display_name.ok_or_else(|| {
        anyhow::anyhow!("missing required field 'displayName' in {}", path.display())
    })?;

    let description = fm.description.ok_or_else(|| {
        anyhow::anyhow!("missing required field 'description' in {}", path.display())
    })?;

    let executor_name = fm.executor.ok_or_else(|| {
        anyhow::anyhow!("missing required field 'executor' in {}", path.display())
    })?;

    let capabilities = fm.capabilities.ok_or_else(|| {
        anyhow::anyhow!(
            "missing required field 'capabilities' in {}",
            path.display()
        )
    })?;

    Ok(ParsedAgent {
        name,
        display_name,
        description,
        executor_name,
        capabilities,
        body,
        temperature: fm.temperature,
    })
}

pub fn parse_directory(dir: &Path) -> Result<Vec<ParsedAgent>> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read directory: {}", dir.display()))?;

    let mut agents = Vec::new();

    for entry in entries {
        let entry = entry
            .with_context(|| format!("failed to read directory entry in {}", dir.display()))?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let agent = parse_file(&path)
                .with_context(|| format!("failed to parse agent file: {}", path.display()))?;
            agents.push(agent);
        }
    }

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

    #[test]
    fn parse_valid_agent_file() {
        let content = r#"---
name: architect
displayName: "Winston the Architect"
description: "Architecture review and design guidance"
executor: bmad/architect
capabilities:
  - architecture-review
  - system-design
---

# Winston the Architect

You are a senior software architect.
"#;
        let f = write_temp_md(content);
        let agent = parse_file(f.path()).expect("should parse valid file");
        assert_eq!(agent.name, "architect");
        assert_eq!(agent.display_name, "Winston the Architect");
        assert_eq!(agent.executor_name, "bmad/architect");
        assert_eq!(
            agent.capabilities,
            vec!["architecture-review", "system-design"]
        );
        assert!(agent.body.contains("Winston the Architect"));
    }

    #[test]
    fn parse_file_missing_frontmatter_delimiter() {
        let content = "No frontmatter here\nJust markdown content.";
        let f = write_temp_md(content);
        let result = parse_file(f.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("frontmatter") || msg.contains("parse"),
            "Error: {}",
            msg
        );
    }

    #[test]
    fn parse_file_malformed_yaml() {
        let content = "---\nname: [unclosed bracket\n---\n# Body";
        let f = write_temp_md(content);
        let result = parse_file(f.path());
        assert!(result.is_err());
    }

    #[test]
    fn parse_file_missing_required_field_name() {
        let content = r#"---
displayName: "Some Agent"
description: "desc"
executor: bmad/test
capabilities:
  - cap
---
Body
"#;
        let f = write_temp_md(content);
        let result = parse_file(f.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("'name'"),
            "Error should mention 'name': {}",
            msg
        );
    }

    #[test]
    fn parse_file_empty_file() {
        let f = write_temp_md("");
        let result = parse_file(f.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("empty"),
            "Error should mention 'empty': {}",
            msg
        );
    }

    #[test]
    fn parse_file_capabilities_as_vec() {
        let content = r#"---
name: qa
displayName: "Quinn the QA"
description: "Quality assurance"
executor: bmad/qa
capabilities:
  - test-generation
  - bug-analysis
  - quality-review
---
Body
"#;
        let f = write_temp_md(content);
        let agent = parse_file(f.path()).expect("should parse");
        assert_eq!(agent.capabilities.len(), 3);
        assert!(agent.capabilities.contains(&"test-generation".to_string()));
    }

    #[test]
    fn parse_file_with_temperature_field() {
        let content = r#"---
name: qa
displayName: "Quinn the QA"
description: "Quality assurance"
executor: bmad/qa
capabilities:
  - test-generation
temperature: 0.7
---
Body text.
"#;
        let f = write_temp_md(content);
        let agent = parse_file(f.path()).expect("should parse agent with temperature");
        assert_eq!(
            agent.temperature,
            Some(0.7_f32),
            "temperature field should be parsed as Some(0.7)"
        );
    }

    #[test]
    fn parse_file_without_temperature_field_yields_none() {
        let content = r#"---
name: architect
displayName: "Winston the Architect"
description: "Architecture review"
executor: bmad/architect
capabilities:
  - architecture-review
---
Body text.
"#;
        let f = write_temp_md(content);
        let agent = parse_file(f.path()).expect("should parse agent without temperature");
        assert_eq!(
            agent.temperature, None,
            "temperature should be None when not present in frontmatter"
        );
    }

    #[test]
    fn test_minimal_valid_agent_parses_successfully() {
        let content = r#"---
name: my-specialist
displayName: "Maya the Specialist"
description: "Provides expert guidance on specialized domain tasks"
executor: bmad/my-specialist
capabilities:
  - domain-analysis
  - task-decomposition
  - recommendation-generation
---

# My Specialist

You are Maya the Specialist.
"#;
        let f = write_temp_md(content);
        let result = parse_file(f.path());
        assert!(
            result.is_ok(),
            "schema minimal example must parse without error"
        );
        let agent = result.unwrap();
        assert!(!agent.name.is_empty(), "name must be non-empty");
        assert!(
            !agent.display_name.is_empty(),
            "display_name must be non-empty"
        );
        assert!(
            !agent.description.is_empty(),
            "description must be non-empty"
        );
        assert!(
            !agent.executor_name.is_empty(),
            "executor_name must be non-empty"
        );
        assert!(
            agent.executor_name.starts_with("bmad/"),
            "executor_name must start with 'bmad/', got: {}",
            agent.executor_name
        );
        assert!(
            agent.capabilities.len() >= 1,
            "capabilities must have at least 1 entry, got: {}",
            agent.capabilities.len()
        );
        assert!(
            !agent.body.is_empty(),
            "body (system prompt) must be non-empty"
        );
    }

    #[test]
    fn parse_directory_with_two_agents() {
        let dir = tempfile::tempdir().unwrap();
        let agent1 = r#"---
name: zebra
displayName: "Zebra Agent"
description: "desc"
executor: bmad/zebra
capabilities:
  - cap-z
---
Body Z
"#;
        let agent2 = r#"---
name: alpha
displayName: "Alpha Agent"
description: "desc"
executor: bmad/alpha
capabilities:
  - cap-a
---
Body A
"#;
        std::fs::write(dir.path().join("zebra.md"), agent1).unwrap();
        std::fs::write(dir.path().join("alpha.md"), agent2).unwrap();

        let agents = parse_directory(dir.path()).expect("should parse directory");
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].name, "alpha");
        assert_eq!(agents[1].name, "zebra");
    }
}
