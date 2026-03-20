use std::time::Instant;

use bmad_types::{AgentMetadata, GenerationParams, SuggestedConfig};
use pulse_plugin_sdk::error::WitPluginError;
use pulse_plugin_sdk::wit_types::{StepConfig, StepResult, TaskInput};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BmadInput {
    pub agent: String,
    #[serde(default)]
    pub prompt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BmadOutputMetadata {
    pub persona: String,
    pub plugin_name: String,
    pub plugin_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BmadOutput {
    pub agent: String,
    pub system_prompt: String,
    pub user_context: String,
    pub suggested_params: Option<GenerationParams>,
    pub suggested_config: Option<SuggestedConfig>,
    pub metadata: BmadOutputMetadata,
}

fn suggested_config_for(executor_name: &str) -> Option<SuggestedConfig> {
    let agent = executor_name.trim_start_matches("bmad/");
    let cfg = match agent {
        "architect" | "bmad-master" => SuggestedConfig {
            model_tier: Some("opus".to_string()),
            max_turns: Some(20),
            permission_mode: Some("plan".to_string()),
            allowed_tools: None,
        },
        "dev" | "developer" => SuggestedConfig {
            model_tier: Some("sonnet".to_string()),
            max_turns: Some(30),
            permission_mode: Some("bypassPermissions".to_string()),
            allowed_tools: None,
        },
        "qa" => SuggestedConfig {
            model_tier: Some("sonnet".to_string()),
            max_turns: Some(15),
            permission_mode: Some("plan".to_string()),
            allowed_tools: None,
        },
        _ => SuggestedConfig {
            model_tier: Some("sonnet".to_string()),
            max_turns: Some(20),
            permission_mode: Some("plan".to_string()),
            allowed_tools: None,
        },
    };
    Some(cfg)
}

pub struct BmadExecutor {
    metadata: &'static AgentMetadata,
    system_prompt: &'static str,
    suggested_params: Option<GenerationParams>,
}

impl BmadExecutor {
    pub fn for_agent(
        metadata: &'static AgentMetadata,
        system_prompt: &'static str,
        suggested_params: Option<GenerationParams>,
    ) -> Self {
        Self {
            metadata,
            system_prompt,
            suggested_params,
        }
    }

    pub fn executor_name(&self) -> &str {
        self.metadata.executor_name
    }

    pub fn execute(
        &self,
        task: TaskInput,
        config: StepConfig,
    ) -> Result<StepResult, WitPluginError> {
        let start = Instant::now();
        let user_context = extract_user_context(&task)?;

        let output = BmadOutput {
            agent: self.metadata.executor_name.to_string(),
            system_prompt: self.system_prompt.to_string(),
            user_context,
            suggested_params: self.suggested_params.clone(),
            suggested_config: suggested_config_for(self.metadata.executor_name),
            metadata: BmadOutputMetadata {
                persona: self.metadata.name.to_string(),
                plugin_name: "bmad-method".to_string(),
                plugin_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let content = serde_json::to_string(&output)
            .map_err(|e| WitPluginError::internal(format!("output serialization failed: {e}")))?;

        let elapsed_ms = start.elapsed().as_millis() as u64;
        Ok(StepResult::success(config.step_id, elapsed_ms).with_content(content))
    }
}

fn extract_user_context(task: &TaskInput) -> Result<String, WitPluginError> {
    let text = if let Some(input_str) = task.input.as_deref() {
        if let Ok(bmad) = serde_json::from_str::<BmadInput>(input_str) {
            bmad.prompt.unwrap_or_else(|| task.description.clone())
        } else {
            input_str.to_string()
        }
    } else {
        task.description.clone()
    };

    if text.trim().is_empty() {
        return Err(WitPluginError::invalid_input("input cannot be empty"));
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated;
    use pulse_plugin_sdk::wit_types::{StepConfig, TaskInput};

    static TEST_META: AgentMetadata = AgentMetadata {
        id: "test-agent",
        name: "test-agent",
        display_name: "Test Agent",
        description: "A test agent for unit testing",
        executor_name: "bmad/test-agent",
        capabilities: &["testing"],
    };

    const TEST_SYSTEM_PROMPT: &str = "You are a test agent. Be thorough and precise.";

    fn test_task(prompt: &str) -> TaskInput {
        TaskInput::new("t-1", prompt).with_input(
            &serde_json::json!({
                "agent": TEST_META.executor_name,
                "prompt": prompt
            })
            .to_string(),
        )
    }

    fn test_config() -> StepConfig {
        StepConfig::new("s-1", "agent")
    }

    fn parse_output(result: &StepResult) -> BmadOutput {
        serde_json::from_str(result.content.as_deref().expect("content must be Some")).unwrap()
    }

    #[test]
    fn executor_returns_output_for_valid_input() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let result = exec.execute(test_task("Review this design."), test_config());
        assert!(result.is_ok());
        let out = parse_output(result.as_ref().unwrap());
        assert!(!out.system_prompt.is_empty());
        assert_eq!(out.user_context, "Review this design.");
    }

    #[test]
    fn executor_returns_error_for_empty_input() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let task = TaskInput::new("t-1", "").with_input("   ");
        let result = exec.execute(task, test_config());
        assert!(matches!(result, Err(ref e) if e.code == "invalid_input"));
    }

    #[test]
    fn executor_name_matches_metadata() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        assert_eq!(exec.executor_name(), "bmad/test-agent");
    }

    #[test]
    fn system_prompt_uses_constant_not_description() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let result = exec
            .execute(test_task("do something"), test_config())
            .unwrap();
        let out = parse_output(&result);
        assert_eq!(out.system_prompt, TEST_SYSTEM_PROMPT);
        assert_ne!(out.system_prompt, TEST_META.description);
    }

    #[test]
    fn valid_agent_dispatch_returns_ok_with_full_system_prompt() {
        let exec = BmadExecutor::for_agent(
            &generated::architect::ARCHITECT,
            generated::architect::SYSTEM_PROMPT,
            generated::architect::suggested_params(),
        );
        assert_eq!(exec.executor_name(), "bmad/architect");
        let task = TaskInput::new("t-1", "review the service mesh architecture").with_input(
            r#"{"agent": "bmad/architect", "prompt": "review the service mesh architecture"}"#,
        );
        let result = exec.execute(task, test_config());
        assert!(result.is_ok(), "Expected Ok from architect executor");
        let out = parse_output(result.as_ref().unwrap());
        assert!(
            out.system_prompt.len() > 100,
            "Expected full SYSTEM_PROMPT (>100 chars), got {} chars",
            out.system_prompt.len()
        );
        assert_eq!(out.user_context, "review the service mesh architecture");
    }

    #[test]
    fn execute_empty_string_returns_invalid_input() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let task = TaskInput::new("t-1", "").with_input("");
        let err = exec
            .execute(task, test_config())
            .expect_err("empty input must return Err");
        assert_eq!(err.code, "invalid_input");
        assert!(
            err.message.contains("empty"),
            "AC3: error message must contain 'empty', got: {}",
            err.message
        );
    }

    #[test]
    fn execute_whitespace_only_returns_invalid_input() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let task = TaskInput::new("t-1", "").with_input("   \t\n  ");
        let err = exec
            .execute(task, test_config())
            .expect_err("whitespace-only input must return Err");
        assert_eq!(err.code, "invalid_input");
        assert!(
            err.message.contains("empty"),
            "AC3: whitespace-only must give same error message, got: {}",
            err.message
        );
    }

    #[test]
    fn executor_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<BmadExecutor>();
    }

    #[test]
    fn user_context_preserved_verbatim() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let input = "Review this API design...";
        let result = exec.execute(test_task(input), test_config()).unwrap();
        let out = parse_output(&result);
        assert_eq!(out.user_context, input);
    }

    #[test]
    fn system_prompt_non_empty_for_all_agents() {
        let agents: &[(&'static bmad_types::AgentMetadata, &'static str)] = &[
            (
                &generated::architect::ARCHITECT,
                generated::architect::SYSTEM_PROMPT,
            ),
            (
                &generated::developer::DEVELOPER,
                generated::developer::SYSTEM_PROMPT,
            ),
            (&generated::pm::PM, generated::pm::SYSTEM_PROMPT),
            (&generated::qa::QA, generated::qa::SYSTEM_PROMPT),
        ];
        for (meta, prompt) in agents {
            let exec = BmadExecutor::for_agent(meta, prompt, None);
            let task = TaskInput::new("t-1", "test input").with_input(
                &serde_json::json!({
                    "agent": meta.executor_name,
                    "prompt": "test input"
                })
                .to_string(),
            );
            let result = exec.execute(task, test_config());
            assert!(result.is_ok(), "unexpected error for agent {}", meta.name);
            let out = parse_output(result.as_ref().unwrap());
            assert!(
                !out.system_prompt.is_empty(),
                "system_prompt must be non-empty for agent {}",
                meta.name
            );
        }
    }

    #[test]
    fn two_outputs_from_same_executor_are_independent() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let r1 = exec
            .execute(test_task("first input"), test_config())
            .unwrap();
        let r2 = exec
            .execute(test_task("second input"), test_config())
            .unwrap();
        let out1 = parse_output(&r1);
        let out2 = parse_output(&r2);
        assert_ne!(out1.user_context, out2.user_context);
        assert_eq!(out1.system_prompt, out2.system_prompt);
    }

    #[test]
    fn outputs_own_independent_strings() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let r1 = exec
            .execute(test_task("input alpha"), test_config())
            .unwrap();
        let r2 = exec
            .execute(test_task("input beta"), test_config())
            .unwrap();
        let out1 = parse_output(&r1);
        let out2 = parse_output(&r2);
        assert_eq!(out1.user_context, "input alpha");
        assert_eq!(out2.user_context, "input beta");
        assert_eq!(out1.system_prompt, out2.system_prompt);
    }

    #[test]
    fn suggested_params_forwarded_from_constructor() {
        use bmad_types::GenerationParams;
        let params = GenerationParams {
            model: None,
            temperature: Some(0.7),
            max_tokens: None,
        };
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, Some(params));
        let result = exec
            .execute(test_task("review this"), test_config())
            .unwrap();
        let out = parse_output(&result);
        let p = out
            .suggested_params
            .expect("expected Some(GenerationParams)");
        assert!((p.temperature.unwrap() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn suggested_params_none_when_not_specified() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let result = exec
            .execute(test_task("review this"), test_config())
            .unwrap();
        let out = parse_output(&result);
        assert!(out.suggested_params.is_none());
    }

    #[test]
    fn architect_system_prompt_contains_persona_keywords() {
        let exec = BmadExecutor::for_agent(
            &generated::architect::ARCHITECT,
            generated::architect::SYSTEM_PROMPT,
            generated::architect::suggested_params(),
        );
        let task = TaskInput::new("t-1", "test input")
            .with_input(r#"{"agent": "bmad/architect", "prompt": "test input"}"#);
        let result = exec.execute(task, test_config()).unwrap();
        let out = parse_output(&result);
        let prompt_lower = out.system_prompt.to_lowercase();
        assert!(
            prompt_lower.contains("architect") || prompt_lower.contains("winston"),
            "Architect SYSTEM_PROMPT must contain 'architect' or 'winston', got: {}",
            &out.system_prompt[..200.min(out.system_prompt.len())]
        );
    }

    #[test]
    fn developer_system_prompt_contains_persona_keywords() {
        let exec = BmadExecutor::for_agent(
            &generated::developer::DEVELOPER,
            generated::developer::SYSTEM_PROMPT,
            generated::developer::suggested_params(),
        );
        let task = TaskInput::new("t-1", "test input")
            .with_input(r#"{"agent": "bmad/dev", "prompt": "test input"}"#);
        let result = exec.execute(task, test_config()).unwrap();
        let out = parse_output(&result);
        let prompt_lower = out.system_prompt.to_lowercase();
        let has_keyword = prompt_lower.contains("concise")
            || prompt_lower.contains("precise")
            || prompt_lower.contains("implementation")
            || prompt_lower.contains("code");
        assert!(
            has_keyword,
            "Developer SYSTEM_PROMPT must contain 'concise', 'precise', 'implementation', or 'code'"
        );
    }

    #[test]
    fn pm_system_prompt_contains_persona_keywords() {
        let exec = BmadExecutor::for_agent(
            &generated::pm::PM,
            generated::pm::SYSTEM_PROMPT,
            generated::pm::suggested_params(),
        );
        let task = TaskInput::new("t-1", "test input")
            .with_input(r#"{"agent": "bmad/pm", "prompt": "test input"}"#);
        let result = exec.execute(task, test_config()).unwrap();
        let out = parse_output(&result);
        let prompt_lower = out.system_prompt.to_lowercase();
        let has_keyword = prompt_lower.contains("why")
            || prompt_lower.contains("requirements")
            || prompt_lower.contains("user value")
            || prompt_lower.contains("validate");
        assert!(
            has_keyword,
            "PM SYSTEM_PROMPT must contain 'why', 'requirements', 'user value', or 'validate'"
        );
    }

    #[test]
    fn qa_system_prompt_contains_persona_keywords() {
        let exec = BmadExecutor::for_agent(
            &generated::qa::QA,
            generated::qa::SYSTEM_PROMPT,
            generated::qa::suggested_params(),
        );
        let task = TaskInput::new("t-1", "test input")
            .with_input(r#"{"agent": "bmad/qa", "prompt": "test input"}"#);
        let result = exec.execute(task, test_config()).unwrap();
        let out = parse_output(&result);
        let prompt_lower = out.system_prompt.to_lowercase();
        let has_keyword = prompt_lower.contains("test")
            || prompt_lower.contains("quality")
            || prompt_lower.contains("verify")
            || prompt_lower.contains("ship");
        assert!(
            has_keyword,
            "QA SYSTEM_PROMPT must contain 'test', 'quality', 'verify', or 'ship'"
        );
    }

    #[test]
    fn bmad_input_rejects_unknown_fields() {
        let result = serde_json::from_str::<BmadInput>(
            r#"{"agent": "bmad/architect", "prompt": "test", "typo": true}"#,
        );
        assert!(
            result.is_err(),
            "deny_unknown_fields must reject unknown keys in task input"
        );
    }

    #[test]
    fn metadata_plugin_name_is_bmad_method() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let result = exec
            .execute(test_task("check plugin name"), test_config())
            .unwrap();
        let out = parse_output(&result);
        assert_eq!(out.metadata.plugin_name, "bmad-method");
    }

    #[test]
    fn suggested_config_for_architect_is_opus() {
        let cfg = suggested_config_for("bmad/architect").unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("opus"));
        assert_eq!(cfg.max_turns, Some(20));
        assert_eq!(cfg.permission_mode.as_deref(), Some("plan"));
    }

    #[test]
    fn suggested_config_for_bmad_master_is_opus() {
        let cfg = suggested_config_for("bmad/bmad-master").unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("opus"));
    }

    #[test]
    fn suggested_config_for_dev_is_sonnet_bypass() {
        let cfg = suggested_config_for("bmad/dev").unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("sonnet"));
        assert_eq!(cfg.permission_mode.as_deref(), Some("bypassPermissions"));
        assert_eq!(cfg.max_turns, Some(30));
    }

    #[test]
    fn suggested_config_for_qa_is_sonnet_plan() {
        let cfg = suggested_config_for("bmad/qa").unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("sonnet"));
        assert_eq!(cfg.max_turns, Some(15));
        assert_eq!(cfg.permission_mode.as_deref(), Some("plan"));
    }
}
