use std::time::Instant;

use bmad_types::{AgentMetadata, GenerationParams, SuggestedConfig};
use pulse_plugin_sdk::error::WitPluginError;
use pulse_plugin_sdk::wit_types::{StepConfig, StepResult, TaskInput};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BmadInput {
    /// Agent name, e.g. "bmad/architect" or "architect".
    /// When called from Pulse workflow, this is extracted from `system_prompt` if not set directly.
    #[serde(default)]
    pub agent: Option<String>,
    /// Optional. If omitted, falls back to `user_prompt_template` or `task.description`.
    #[serde(default)]
    pub prompt: Option<String>,
    /// Pulse workflow passes system_prompt — extract agent name from "bmad/<name>" pattern.
    #[serde(default)]
    pub system_prompt: Option<String>,
    /// Pulse workflow passes user_prompt_template — use as prompt fallback.
    #[serde(default)]
    pub user_prompt_template: Option<String>,
}

impl BmadInput {
    /// Returns the agent name normalized with the `bmad/` prefix.
    /// Resolves agent from: explicit `agent` field, or `system_prompt` pattern "bmad/<name>".
    /// Returns None if no agent can be resolved.
    pub fn normalized_agent(&self) -> Option<String> {
        // Try explicit agent field first
        if let Some(ref agent) = self.agent {
            if !agent.is_empty() {
                return Some(if agent.starts_with("bmad/") {
                    agent.clone()
                } else {
                    format!("bmad/{}", agent)
                });
            }
        }
        // Extract from system_prompt: look for "bmad/<name>" pattern
        if let Some(ref prompt) = self.system_prompt {
            if let Some(idx) = prompt.find("bmad/") {
                let after = &prompt[idx + 5..];
                let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
                if !name.is_empty() {
                    return Some(format!("bmad/{}", name));
                }
            }
        }
        None
    }

    /// Resolve the user prompt from available fields.
    pub fn resolved_prompt(&self) -> Option<String> {
        self.prompt.clone()
            .or_else(|| self.user_prompt_template.clone())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BmadOutputMetadata {
    pub persona: String,
    pub plugin_name: String,
    pub plugin_version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BmadOutput {
    pub schema_version: String,
    pub agent: String,
    pub system_prompt: String,
    /// Raw task input. Not sanitized — consumer must sanitize before rendering
    /// in injection-sensitive contexts.
    pub user_context: String,
    pub suggested_params: Option<GenerationParams>,
    pub suggested_config: Option<SuggestedConfig>,
    pub metadata: BmadOutputMetadata,
}

pub struct BmadExecutor {
    metadata: &'static AgentMetadata,
    system_prompt: &'static str,
    suggested_params: Option<GenerationParams>,
    suggested_config: Option<SuggestedConfig>,
}

impl BmadExecutor {
    pub fn for_agent(
        metadata: &'static AgentMetadata,
        system_prompt: &'static str,
        suggested_params: Option<GenerationParams>,
        suggested_config: Option<SuggestedConfig>,
    ) -> Self {
        Self {
            metadata,
            system_prompt,
            suggested_params,
            suggested_config,
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
            schema_version: "1.1".to_string(),
            agent: self.metadata.executor_name.to_string(),
            system_prompt: self.system_prompt.to_string(),
            user_context,
            suggested_params: self.suggested_params.clone(),
            suggested_config: self.suggested_config.clone(),
            metadata: BmadOutputMetadata {
                persona: self.metadata.name.to_string(),
                plugin_name: "bmad-method".to_string(),
                plugin_version: env!("CARGO_PKG_VERSION").to_string(),
                capabilities: self.metadata.capabilities.iter().map(|s| s.to_string()).collect(),
            },
        };

        let content = serde_json::to_string(&output)
            .map_err(|e| WitPluginError::internal(format!("output serialization failed: {e}")))?;

        let elapsed_ms = start.elapsed().as_millis() as u64;
        Ok(StepResult::success(config.step_id, elapsed_ms).with_content(content))
    }
}

/// Maximum input size in bytes (128KB). Prevents oversized payloads from causing
/// excessive memory allocation, especially in WASM environments.
const MAX_INPUT_LEN: usize = 131072;

fn extract_user_context(task: &TaskInput) -> Result<String, WitPluginError> {
    let text = if let Some(ref input_val) = task.input {
        // Try to deserialize the Value as BmadInput
        if let Ok(bmad) = serde_json::from_value::<BmadInput>(input_val.clone()) {
            bmad.resolved_prompt().unwrap_or_else(|| {
                warn!("prompt field missing in BmadInput, falling back to task.description");
                task.description.clone()
            })
        } else if let Some(s) = input_val.as_str() {
            s.to_string()
        } else {
            input_val.to_string()
        }
    } else {
        task.description.clone()
    };

    if text.len() > MAX_INPUT_LEN {
        return Err(WitPluginError::invalid_input(format!(
            "input exceeds maximum length of {} bytes",
            MAX_INPUT_LEN
        )));
    }

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
        name: "test-agent",
        display_name: "Test Agent",
        description: "A test agent for unit testing",
        executor_name: "bmad/test-agent",
        capabilities: &["testing"],
    };

    const TEST_SYSTEM_PROMPT: &str = "You are a test agent. Be thorough and precise.";

    fn test_task(prompt: &str) -> TaskInput {
        TaskInput::new("t-1", prompt).with_input(
            serde_json::json!({
                "agent": TEST_META.executor_name,
                "prompt": prompt
            }),
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
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
        let result = exec.execute(test_task("Review this design."), test_config());
        assert!(result.is_ok());
        let out = parse_output(result.as_ref().unwrap());
        assert!(!out.system_prompt.is_empty());
        assert_eq!(out.user_context, "Review this design.");
    }

    #[test]
    fn executor_returns_error_for_empty_input() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
        let task = TaskInput::new("t-1", "").with_input(serde_json::json!("   "));
        let result = exec.execute(task, test_config());
        assert!(matches!(result, Err(ref e) if e.code == "invalid_input"));
    }

    #[test]
    fn executor_name_matches_metadata() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
        assert_eq!(exec.executor_name(), "bmad/test-agent");
    }

    #[test]
    fn system_prompt_uses_constant_not_description() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
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
            generated::architect::suggested_config(),
        );
        assert_eq!(exec.executor_name(), "bmad/architect");
        let task = TaskInput::new("t-1", "review the service mesh architecture").with_input(
            serde_json::json!({"agent": "bmad/architect", "prompt": "review the service mesh architecture"}),
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
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
        let task = TaskInput::new("t-1", "").with_input(serde_json::json!(""));
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
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
        let task = TaskInput::new("t-1", "").with_input(serde_json::json!("   \t\n  "));
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
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
        let input = "Review this API design...";
        let result = exec.execute(test_task(input), test_config()).unwrap();
        let out = parse_output(&result);
        assert_eq!(out.user_context, input);
    }

    #[test]
    fn system_prompt_non_empty_for_all_agents() {
        for (meta, prompt, _params, _config) in generated::all_agent_entries() {
            assert!(
                !prompt.is_empty(),
                "system_prompt must be non-empty for agent {}",
                meta.name
            );
        }
    }

    #[test]
    fn two_outputs_from_same_executor_are_independent() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
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
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
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
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, Some(params), None);
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
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
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
            generated::architect::suggested_config(),
        );
        let task = TaskInput::new("t-1", "test input")
            .with_input(serde_json::json!({"agent": "bmad/architect", "prompt": "test input"}));
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
            generated::developer::suggested_config(),
        );
        let task = TaskInput::new("t-1", "test input")
            .with_input(serde_json::json!({"agent": "bmad/dev", "prompt": "test input"}));
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
            generated::pm::suggested_config(),
        );
        let task = TaskInput::new("t-1", "test input")
            .with_input(serde_json::json!({"agent": "bmad/pm", "prompt": "test input"}));
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
            generated::qa::suggested_config(),
        );
        let task = TaskInput::new("t-1", "test input")
            .with_input(serde_json::json!({"agent": "bmad/qa", "prompt": "test input"}));
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
    fn input_at_size_limit_succeeds() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
        let large_prompt = "x".repeat(MAX_INPUT_LEN);
        let task = TaskInput::new("t-1", "test").with_input(
            serde_json::json!({"agent": TEST_META.executor_name, "prompt": large_prompt}),
        );
        let result = exec.execute(task, test_config());
        assert!(result.is_ok(), "input at exactly MAX_INPUT_LEN should succeed");
    }

    #[test]
    fn input_over_size_limit_returns_error() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
        let large_prompt = "x".repeat(MAX_INPUT_LEN + 1);
        let task = TaskInput::new("t-1", "test").with_input(
            serde_json::json!({"agent": TEST_META.executor_name, "prompt": large_prompt}),
        );
        let err = exec.execute(task, test_config()).expect_err("over-limit input must fail");
        assert_eq!(err.code, "invalid_input");
        assert!(
            err.message.contains("131072"),
            "error must contain the limit value, got: {}",
            err.message
        );
    }

    #[test]
    fn metadata_plugin_name_is_bmad_method() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
        let result = exec
            .execute(test_task("check plugin name"), test_config())
            .unwrap();
        let out = parse_output(&result);
        assert_eq!(out.metadata.plugin_name, "bmad-method");
    }

    #[test]
    fn generated_config_architect_is_opus() {
        let cfg = generated::architect::suggested_config().unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("opus"));
        assert_eq!(cfg.max_turns, Some(20));
        assert_eq!(cfg.permission_mode.as_deref(), Some("plan"));
    }

    #[test]
    fn generated_config_bmad_master_is_opus() {
        let cfg = generated::bmad_master::suggested_config().unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("opus"));
    }

    #[test]
    fn generated_config_dev_is_sonnet_bypass() {
        let cfg = generated::developer::suggested_config().unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("sonnet"));
        assert_eq!(cfg.permission_mode.as_deref(), Some("bypassPermissions"));
        assert_eq!(cfg.max_turns, Some(30));
    }

    #[test]
    fn generated_config_qa_is_sonnet_plan() {
        let cfg = generated::qa::suggested_config().unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("sonnet"));
        assert_eq!(cfg.max_turns, Some(15));
        assert_eq!(cfg.permission_mode.as_deref(), Some("plan"));
    }

    // Story 8.1: keyword tests for remaining 8 agents
    #[test]
    fn analyst_system_prompt_contains_persona_keywords() {
        let prompt = generated::analyst::SYSTEM_PROMPT.to_lowercase();
        assert!(
            prompt.contains("analyst") || prompt.contains("market") || prompt.contains("research") || prompt.contains("requirements"),
            "Analyst SYSTEM_PROMPT must contain relevant keywords"
        );
    }

    #[test]
    fn bmad_master_system_prompt_contains_persona_keywords() {
        let prompt = generated::bmad_master::SYSTEM_PROMPT.to_lowercase();
        assert!(
            prompt.contains("orchestrat") || prompt.contains("workflow") || prompt.contains("master") || prompt.contains("knowledge"),
            "BmadMaster SYSTEM_PROMPT must contain relevant keywords"
        );
    }

    #[test]
    fn devops_system_prompt_contains_persona_keywords() {
        let prompt = generated::devops::SYSTEM_PROMPT.to_lowercase();
        assert!(
            prompt.contains("pipeline") || prompt.contains("infrastructure") || prompt.contains("deployment") || prompt.contains("ci"),
            "DevOps SYSTEM_PROMPT must contain relevant keywords"
        );
    }

    #[test]
    fn quick_flow_solo_dev_system_prompt_contains_persona_keywords() {
        let prompt = generated::quick_flow_solo_dev::SYSTEM_PROMPT.to_lowercase();
        assert!(
            prompt.contains("quick") || prompt.contains("lean") || prompt.contains("spec") || prompt.contains("barry"),
            "QuickFlowSoloDev SYSTEM_PROMPT must contain relevant keywords"
        );
    }

    #[test]
    fn scrum_master_system_prompt_contains_persona_keywords() {
        let prompt = generated::scrum_master::SYSTEM_PROMPT.to_lowercase();
        assert!(
            prompt.contains("sprint") || prompt.contains("agile") || prompt.contains("scrum") || prompt.contains("ceremony"),
            "ScrumMaster SYSTEM_PROMPT must contain relevant keywords"
        );
    }

    #[test]
    fn security_system_prompt_contains_persona_keywords() {
        let prompt = generated::security::SYSTEM_PROMPT.to_lowercase();
        assert!(
            prompt.contains("threat") || prompt.contains("security") || prompt.contains("vulnerabilit") || prompt.contains("defense"),
            "Security SYSTEM_PROMPT must contain relevant keywords"
        );
    }

    #[test]
    fn tech_writer_system_prompt_contains_persona_keywords() {
        let prompt = generated::tech_writer::SYSTEM_PROMPT.to_lowercase();
        assert!(
            prompt.contains("documentation") || prompt.contains("clarity") || prompt.contains("technical writ"),
            "TechWriter SYSTEM_PROMPT must contain relevant keywords"
        );
    }

    #[test]
    fn ux_designer_system_prompt_contains_persona_keywords() {
        let prompt = generated::ux_designer::SYSTEM_PROMPT.to_lowercase();
        assert!(
            prompt.contains("user experience") || prompt.contains("ux") || prompt.contains("design") || prompt.contains("empathy"),
            "UxDesigner SYSTEM_PROMPT must contain relevant keywords"
        );
    }

    // Story 8.3: suggested_config wildcard/fallback tests
    #[test]
    fn generated_config_analyst_is_sonnet_default() {
        let cfg = generated::analyst::suggested_config().unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("sonnet"));
        assert_eq!(cfg.max_turns, Some(20));
        assert_eq!(cfg.permission_mode.as_deref(), Some("plan"));
    }

    #[test]
    fn generated_config_devops_is_sonnet_default() {
        let cfg = generated::devops::suggested_config().unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("sonnet"));
        assert_eq!(cfg.max_turns, Some(20));
        assert_eq!(cfg.permission_mode.as_deref(), Some("plan"));
    }

    #[test]
    fn generated_config_security_is_sonnet_default() {
        let cfg = generated::security::suggested_config().unwrap();
        assert_eq!(cfg.model_tier.as_deref(), Some("sonnet"));
        assert_eq!(cfg.max_turns, Some(20));
        assert_eq!(cfg.permission_mode.as_deref(), Some("plan"));
    }

    #[test]
    fn all_agents_have_suggested_config() {
        for (meta, _prompt, _params, config) in generated::all_agent_entries() {
            assert!(
                config.is_some(),
                "agent '{}' must have a suggested_config",
                meta.name
            );
        }
    }

    // Story 9.1: schema_version tests
    #[test]
    fn output_contains_schema_version() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None, None);
        let result = exec.execute(test_task("test"), test_config()).unwrap();
        let out = parse_output(&result);
        assert_eq!(out.schema_version, "1.1");
    }

    // Story 9.2: capabilities in output metadata
    #[test]
    fn output_metadata_contains_capabilities() {
        let exec = BmadExecutor::for_agent(
            &generated::architect::ARCHITECT,
            generated::architect::SYSTEM_PROMPT,
            generated::architect::suggested_params(),
            generated::architect::suggested_config(),
        );
        let task = TaskInput::new("t-1", "test input")
            .with_input(serde_json::json!({"agent": "bmad/architect", "prompt": "test input"}));
        let result = exec.execute(task, test_config()).unwrap();
        let out = parse_output(&result);
        assert!(
            !out.metadata.capabilities.is_empty(),
            "architect must have at least 1 capability in output"
        );
        assert!(
            out.metadata.capabilities.contains(&"architecture-review".to_string()),
            "architect capabilities must contain 'architecture-review'"
        );
    }

    #[test]
    fn all_agents_have_capabilities_in_output() {
        for (meta, prompt, params, config) in generated::all_agent_entries() {
            let exec = BmadExecutor::for_agent(meta, prompt, params, config);
            let task = TaskInput::new("t-1", "test").with_input(
                serde_json::json!({"agent": meta.executor_name, "prompt": "test"}),
            );
            let result = exec.execute(task, test_config()).unwrap();
            let out = parse_output(&result);
            assert!(
                !out.metadata.capabilities.is_empty(),
                "agent '{}' must have at least 1 capability in output",
                meta.name
            );
        }
    }
}
