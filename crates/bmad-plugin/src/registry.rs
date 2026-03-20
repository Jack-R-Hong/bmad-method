use std::collections::HashMap;
use std::sync::OnceLock;

#[cfg(test)]
use bmad_types::BmadError;
use bmad_types::{AgentMetadata, VerificationResult};
use pulse_plugin_sdk::wit_types::{StepConfig, TaskInput};

static GLOBAL_REGISTRY: OnceLock<AgentRegistry> = OnceLock::new();

fn global_registry() -> &'static AgentRegistry {
    GLOBAL_REGISTRY.get_or_init(AgentRegistry::new)
}

pub fn list_agents() -> &'static [AgentMetadata] {
    global_registry().list_agents()
}

pub fn find_agent(executor_name: &str) -> Option<&'static AgentMetadata> {
    global_registry().find_agent(executor_name)
}

pub fn verify_all_agents() -> Vec<VerificationResult> {
    let entries = crate::generated::all_agent_entries();
    entries
        .into_iter()
        .map(|(meta, prompt, params)| {
            let executor = crate::executor::BmadExecutor::for_agent(meta, prompt, params);
            let task = TaskInput::new("health-check", "ping").with_input("ping");
            let config = StepConfig::new("health-check", "agent");
            match executor.execute(task, config) {
                Ok(_) => VerificationResult {
                    executor_name: meta.executor_name.to_string(),
                    passed: true,
                    failure_reason: None,
                },
                Err(e) => VerificationResult {
                    executor_name: meta.executor_name.to_string(),
                    passed: false,
                    failure_reason: Some(e.to_string()),
                },
            }
        })
        .collect()
}

pub struct AgentRegistry {
    agents: HashMap<&'static str, &'static AgentMetadata>,
    sorted: Vec<AgentMetadata>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let all = crate::generated::all_agents();
        let mut agents: HashMap<&'static str, &'static AgentMetadata> =
            HashMap::with_capacity(all.len());
        for meta in &all {
            agents.insert(meta.executor_name, *meta);
        }
        let mut sorted: Vec<AgentMetadata> = all.into_iter().copied().collect();
        sorted.sort_by_key(|m| m.executor_name);
        Self { agents, sorted }
    }

    #[allow(dead_code)]
    pub fn find_agent(&self, executor_name: &str) -> Option<&'static AgentMetadata> {
        self.agents.get(executor_name).copied()
    }

    #[allow(dead_code)]
    pub fn list_agents(&self) -> &[AgentMetadata] {
        &self.sorted
    }

    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.agents.len()
    }

    #[cfg(test)]
    pub fn dispatch(
        &self,
        executor_name: &str,
        input: &str,
    ) -> Result<bmad_types::AgentOutput, BmadError> {
        if executor_name.trim().is_empty() {
            return Err(BmadError::InvalidInput(
                "executor name cannot be empty".to_string(),
            ));
        }
        match self.find_agent(executor_name) {
            Some(meta) => Ok(bmad_types::AgentOutput {
                system_prompt: meta.description.to_string(),
                user_context: input.to_string(),
                suggested_params: None,
            }),
            None => Err(BmadError::AgentNotFound(executor_name.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_agents() {
        let registry = AgentRegistry::new();
        assert!(registry.count() >= 1, "Registry must have at least 1 agent");
    }

    #[test]
    fn find_known_agent_returns_some() {
        assert!(
            find_agent("bmad/architect").is_some(),
            "Expected to find 'bmad/architect' but got None"
        );
    }

    #[test]
    fn find_developer_agent_returns_some() {
        assert!(
            find_agent("bmad/dev").is_some(),
            "Expected to find 'bmad/dev' but got None"
        );
    }

    #[test]
    fn find_pm_agent_returns_some() {
        assert!(
            find_agent("bmad/pm").is_some(),
            "Expected to find 'bmad/pm' but got None"
        );
    }

    #[test]
    fn find_qa_agent_returns_some() {
        assert!(
            find_agent("bmad/qa").is_some(),
            "Expected to find 'bmad/qa' but got None"
        );
    }

    #[test]
    fn find_unknown_agent_returns_none() {
        assert!(find_agent("bmad/nonexistent").is_none());
    }

    #[test]
    fn find_empty_name_returns_none() {
        assert!(find_agent("").is_none());
    }

    #[test]
    fn find_agent_is_case_sensitive() {
        assert!(
            find_agent("bmad/Architect").is_none(),
            "find_agent must be case-sensitive: 'bmad/Architect' should return None"
        );
    }

    #[test]
    fn all_registered_names_have_bmad_prefix() {
        for agent in list_agents() {
            assert!(
                agent.executor_name.starts_with("bmad/"),
                "Agent '{}' has invalid executor name: {}",
                agent.name,
                agent.executor_name
            );
        }
    }

    #[test]
    fn list_agents_returns_sorted_alphabetical() {
        let agents = list_agents();
        for i in 1..agents.len() {
            assert!(
                agents[i - 1].executor_name <= agents[i].executor_name,
                "Agents not sorted: '{}' should come before '{}'",
                agents[i - 1].executor_name,
                agents[i].executor_name
            );
        }
    }

    #[test]
    fn dispatch_valid_agent_returns_ok() {
        let registry = AgentRegistry::new();
        let result = registry.dispatch("bmad/architect", "design a system");
        assert!(result.is_ok(), "Expected Ok for known agent");
    }

    #[test]
    fn dispatch_unknown_agent_returns_agent_not_found() {
        let registry = AgentRegistry::new();
        let result = registry.dispatch("bmad/nonexistent", "some task");
        assert!(
            matches!(&result, Err(BmadError::AgentNotFound(name)) if name == "bmad/nonexistent"),
            "Expected AgentNotFound with the unrecognized name, got: {:?}",
            result
        );
    }

    #[test]
    fn dispatch_empty_name_returns_invalid_input() {
        let registry = AgentRegistry::new();
        assert!(matches!(
            registry.dispatch("", "task"),
            Err(BmadError::InvalidInput(_))
        ));
    }

    #[test]
    fn dispatch_whitespace_name_returns_invalid_input() {
        let registry = AgentRegistry::new();
        assert!(matches!(
            registry.dispatch("   ", "task"),
            Err(BmadError::InvalidInput(_))
        ));
    }

    #[test]
    fn agent_count_matches_source_files() {
        const EXPECTED_AGENT_COUNT: usize = 12;
        assert_eq!(
            list_agents().len(),
            EXPECTED_AGENT_COUNT,
            "Agent count mismatch: update EXPECTED_AGENT_COUNT or add missing .md files in agents/"
        );
    }

    #[test]
    fn all_agent_fields_non_empty() {
        for agent in list_agents() {
            assert!(
                !agent.name.is_empty(),
                "name is empty for executor '{}'",
                agent.executor_name
            );
            assert!(
                !agent.display_name.is_empty(),
                "display_name is empty for executor '{}'",
                agent.executor_name
            );
            assert!(
                !agent.description.is_empty(),
                "description is empty for executor '{}'",
                agent.executor_name
            );
            assert!(
                !agent.capabilities.is_empty(),
                "capabilities is empty for executor '{}'",
                agent.executor_name
            );
        }
    }

    #[test]
    fn no_duplicate_executor_names() {
        let all = crate::generated::all_agents();
        let mut names: Vec<&str> = all.iter().map(|a| a.executor_name).collect();
        let original_len = names.len();
        names.sort();
        names.dedup();
        assert_eq!(
            names.len(),
            original_len,
            "Duplicate executor names found in generated agent list — {} source agents reduced to {} unique names",
            original_len,
            names.len()
        );
    }

    #[test]
    fn verify_all_agents_returns_results_for_all_registered_agents() {
        let results = super::verify_all_agents();
        assert!(!results.is_empty());
        for r in &results {
            assert!(r.executor_name.starts_with("bmad/"));
        }
    }

    #[test]
    fn verify_all_agents_count_matches_registry() {
        let results = super::verify_all_agents();
        assert_eq!(
            results.len(),
            list_agents().len(),
            "verify_all_agents must return one result per registered agent"
        );
    }

    #[test]
    fn verify_all_agents_all_pass_with_valid_input() {
        let results = super::verify_all_agents();
        let failures: Vec<_> = results.iter().filter(|r| !r.passed).collect();
        assert!(failures.is_empty(), "Unexpected failures: {:?}", failures);
    }

    #[test]
    fn verify_all_agents_passed_true_implies_no_failure_reason() {
        let results = super::verify_all_agents();
        for r in &results {
            if r.passed {
                assert!(
                    r.failure_reason.is_none(),
                    "Agent '{}': passed=true but failure_reason is Some({:?})",
                    r.executor_name,
                    r.failure_reason
                );
            } else {
                assert!(
                    r.failure_reason.is_some(),
                    "Agent '{}': passed=false but failure_reason is None",
                    r.executor_name
                );
            }
        }
    }

    #[test]
    fn all_executor_names_follow_bmad_namespace() {
        for agent in list_agents() {
            assert!(
                agent.executor_name.starts_with("bmad/"),
                "executor_name '{}' does not start with 'bmad/'",
                agent.executor_name
            );
            let identifier = &agent.executor_name["bmad/".len()..];
            assert!(
                !identifier.is_empty(),
                "executor identifier must not be empty for '{}'",
                agent.executor_name
            );
            assert!(
                identifier
                    .chars()
                    .all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '-'),
                "executor identifier '{}' contains invalid characters (must be lowercase, digits, hyphens only)",
                identifier
            );
        }
    }
}
