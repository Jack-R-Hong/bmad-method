use crate::codegen::templates::{generate_agent_file, generate_mod_file, to_snake_case};
use crate::parser::ParsedAgent;
use anyhow::{Context, Result};
use std::path::Path;

pub fn write_agent_files(agents: &[ParsedAgent], output_dir: &Path) -> Result<()> {
    if output_dir.exists() {
        for entry in std::fs::read_dir(output_dir)
            .with_context(|| format!("failed to read output dir: {}", output_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                std::fs::remove_file(&path)
                    .with_context(|| format!("failed to remove stale file: {}", path.display()))?;
            }
        }
    } else {
        std::fs::create_dir_all(output_dir)
            .with_context(|| format!("failed to create output dir: {}", output_dir.display()))?;
    }

    let timestamp = chrono::Utc::now().to_rfc3339();

    for agent in agents {
        let filename = format!("{}.rs", to_snake_case(&agent.name));
        let content = generate_agent_file(agent, &timestamp);
        let path = output_dir.join(&filename);
        std::fs::write(&path, content)
            .with_context(|| format!("failed to write: {}", path.display()))?;
    }

    let mod_content = generate_mod_file(agents);
    let mod_path = output_dir.join("mod.rs");
    std::fs::write(&mod_path, mod_content)
        .with_context(|| format!("failed to write mod.rs: {}", mod_path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_agent(name: &str) -> ParsedAgent {
        ParsedAgent {
            name: name.to_string(),
            display_name: format!("{} Display", name),
            description: format!("{} description", name),
            executor_name: format!("bmad/{}", name),
            capabilities: vec!["cap".to_string()],
            body: format!("# {}\n\nBody.", name),
            temperature: None,
            model_tier: None,
            max_turns: None,
            permission_mode: None,
        }
    }

    #[test]
    fn write_agent_files_creates_rs_files() {
        let dir = tempdir().unwrap();
        let agents = vec![make_agent("architect"), make_agent("developer")];
        write_agent_files(&agents, dir.path()).unwrap();

        assert!(dir.path().join("architect.rs").exists());
        assert!(dir.path().join("developer.rs").exists());
        assert!(dir.path().join("mod.rs").exists());
    }

    #[test]
    fn write_agent_files_cleans_stale_files() {
        let dir = tempdir().unwrap();
        let stale_path = dir.path().join("old_stale.rs");
        std::fs::write(&stale_path, "stale content").unwrap();
        assert!(stale_path.exists());

        let agents = vec![make_agent("architect")];
        write_agent_files(&agents, dir.path()).unwrap();

        assert!(!stale_path.exists(), "stale file should have been removed");
        assert!(dir.path().join("architect.rs").exists());
    }

    #[test]
    fn write_agent_files_creates_output_dir_if_missing() {
        let dir = tempdir().unwrap();
        let output_dir = dir.path().join("new_subdir");
        assert!(!output_dir.exists());

        let agents = vec![make_agent("qa")];
        write_agent_files(&agents, &output_dir).unwrap();

        assert!(output_dir.exists());
        assert!(output_dir.join("qa.rs").exists());
    }

    #[test]
    fn write_agent_files_tech_writer_uses_snake_case_filename() {
        let dir = tempdir().unwrap();
        let agents = vec![make_agent("tech-writer")];
        write_agent_files(&agents, dir.path()).unwrap();

        assert!(dir.path().join("tech_writer.rs").exists());
        assert!(!dir.path().join("tech-writer.rs").exists());
    }
}
