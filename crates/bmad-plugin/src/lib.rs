pub mod executor;
pub mod generated;
pub mod registry;

use pulse_plugin_sdk::error::WitPluginError;
use pulse_plugin_sdk::wit_traits::{PluginLifecycle, StepExecutorPlugin};
use pulse_plugin_sdk::wit_types::{PluginInfo, StepConfig, StepResult, TaskInput};
use tracing::{error, info};

use executor::BmadInput;

#[derive(Default)]
pub struct BmadMethodPlugin;

impl PluginLifecycle for BmadMethodPlugin {
    fn get_info(&self) -> PluginInfo {
        PluginInfo::new("bmad-method", env!("CARGO_PKG_VERSION")).with_description(
            "BMAD AI team — 12 specialized agents (architect, dev, pm, qa, …) as a single step executor",
        )
    }

    fn health_check(&self) -> bool {
        let healthy = !registry::list_agents().is_empty();
        if healthy {
            info!(
                plugin = "bmad-method",
                status = "healthy",
                version = env!("CARGO_PKG_VERSION"),
                "WASM plugin health check passed"
            );
        } else {
            error!(
                plugin = "bmad-method",
                status = "error",
                reason = "no agents registered",
                "WASM plugin health check failed"
            );
        }
        healthy
    }
}

impl StepExecutorPlugin for BmadMethodPlugin {
    fn execute(&self, task: TaskInput, config: StepConfig) -> Result<StepResult, WitPluginError> {
        let input_str = task.input.as_deref().ok_or_else(|| {
            WitPluginError::invalid_input(
                "task input is required; send JSON {\"agent\": \"bmad/architect\", \"prompt\": \"...\"}",
            )
        })?;

        let bmad_input: BmadInput = serde_json::from_str(input_str)
            .map_err(|e| WitPluginError::invalid_input(format!("invalid BMAD input JSON: {e}")))?;

        let entries = generated::all_agent_entries();
        let (meta, prompt, params) = entries
            .into_iter()
            .find(|(m, _, _)| m.executor_name == bmad_input.agent)
            .ok_or_else(|| {
                let available: Vec<&str> = generated::all_agents()
                    .iter()
                    .map(|a| a.executor_name)
                    .collect();
                WitPluginError::not_found(format!(
                    "Unknown agent persona: {}. Available: [{}]",
                    bmad_input.agent,
                    available.join(", ")
                ))
            })?;

        let exec = executor::BmadExecutor::for_agent(meta, prompt, params);
        exec.execute(task, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulse_plugin_sdk::wit_types::{StepConfig, TaskInput};

    #[test]
    fn plugin_health_check_returns_true() {
        let plugin = BmadMethodPlugin;
        assert!(plugin.health_check());
    }

    #[test]
    fn plugin_get_info_returns_correct_name() {
        let plugin = BmadMethodPlugin;
        let info = plugin.get_info();
        assert_eq!(info.name, "bmad-method");
        assert!(!info.version.is_empty());
        assert!(info.description.is_some());
    }

    #[test]
    fn all_agents_returns_at_least_one() {
        let agents = generated::all_agents();
        assert!(!agents.is_empty(), "No agents registered");
    }

    #[test]
    fn all_executor_names_start_with_bmad_prefix() {
        let agents = generated::all_agents();
        for agent in agents {
            assert!(
                agent.executor_name.starts_with("bmad/"),
                "Executor name '{}' must start with 'bmad/'",
                agent.executor_name
            );
        }
    }

    #[test]
    fn execute_missing_input_returns_invalid_input() {
        let plugin = BmadMethodPlugin;
        let task = TaskInput::new("t1", "test task");
        let config = StepConfig::new("s1", "agent");
        let err = plugin.execute(task, config).unwrap_err();
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn execute_invalid_json_returns_invalid_input() {
        let plugin = BmadMethodPlugin;
        let task = TaskInput::new("t1", "test").with_input("not json at all");
        let config = StepConfig::new("s1", "agent");
        let err = plugin.execute(task, config).unwrap_err();
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn execute_unknown_agent_returns_not_found() {
        let plugin = BmadMethodPlugin;
        let task = TaskInput::new("t1", "test")
            .with_input(r#"{"agent": "bmad/nonexistent", "prompt": "test"}"#);
        let config = StepConfig::new("s1", "agent");
        let err = plugin.execute(task, config).unwrap_err();
        assert_eq!(err.code, "not_found");
        assert!(
            err.message
                .contains("Unknown agent persona: bmad/nonexistent"),
            "error must name the unknown agent, got: {}",
            err.message
        );
        assert!(
            err.message.contains("Available:"),
            "error must list available agents, got: {}",
            err.message
        );
    }

    #[test]
    fn execute_architect_returns_success() {
        let plugin = BmadMethodPlugin;
        let task = TaskInput::new("t1", "Design a system").with_input(
            r#"{"agent": "bmad/architect", "prompt": "Design a microservices architecture"}"#,
        );
        let config = StepConfig::new("s1", "agent");
        let result = plugin.execute(task, config).unwrap();
        assert_eq!(result.status, "success");
        assert_eq!(result.step_id, "s1");
        assert!(result.content.is_some());
    }

    #[test]
    fn execute_content_is_valid_json_with_system_prompt() {
        let plugin = BmadMethodPlugin;
        let task = TaskInput::new("t1", "test")
            .with_input(r#"{"agent": "bmad/architect", "prompt": "Design a system"}"#);
        let config = StepConfig::new("s1", "agent");
        let result = plugin.execute(task, config).unwrap();
        let content: serde_json::Value =
            serde_json::from_str(result.content.as_deref().unwrap()).unwrap();
        assert!(!content["system_prompt"].as_str().unwrap_or("").is_empty());
        assert_eq!(content["agent"].as_str().unwrap(), "bmad/architect");
        assert_eq!(content["user_context"].as_str().unwrap(), "Design a system");
        assert_eq!(
            content["metadata"]["persona"].as_str().unwrap(),
            "architect"
        );
        assert_eq!(
            content["metadata"]["plugin_name"].as_str().unwrap(),
            "bmad-method"
        );
        assert!(!content["metadata"]["plugin_version"]
            .as_str()
            .unwrap_or("")
            .is_empty());
        assert!(content["suggested_config"].is_object());
    }

    #[test]
    fn execute_all_agents_return_success() {
        let plugin = BmadMethodPlugin;
        let agents = generated::all_agents();
        for agent in agents {
            let input = format!(
                r#"{{"agent": "{}", "prompt": "test input for agent"}}"#,
                agent.executor_name
            );
            let task = TaskInput::new("t1", "test").with_input(&input);
            let config = StepConfig::new("s1", "agent");
            let result = plugin.execute(task, config);
            assert!(
                result.is_ok(),
                "execute failed for agent '{}': {:?}",
                agent.executor_name,
                result.err()
            );
        }
    }
}
