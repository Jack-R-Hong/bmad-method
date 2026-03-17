// crates/bmad-types/src/output.rs
use serde::{Deserialize, Serialize};

/// Optional generation parameters an agent can suggest to Pulse.
/// Pulse is free to ignore these — the plugin does not own LLM execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    /// Preferred model identifier (e.g., "gpt-4", "claude-3-opus")
    pub model: Option<String>,
    /// Sampling temperature, typically 0.0–2.0
    pub temperature: Option<f32>,
    /// Maximum tokens for the response
    pub max_tokens: Option<u32>,
}

/// Structured output returned by a BMAD agent executor.
/// Pulse owns LLM execution — this struct carries prompt data only.
#[derive(Debug, Clone)]
pub struct AgentOutput {
    /// The agent's persona/role instructions for the LLM system prompt
    pub system_prompt: String,
    /// Task input as passed by the Pulse workflow, forwarded to user turn
    pub user_context: String,
    /// Optional generation parameters the agent suggests to Pulse
    pub suggested_params: Option<GenerationParams>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_output_with_no_params() {
        let output = AgentOutput {
            system_prompt: "You are a senior architect.".to_string(),
            user_context: "Review this design.".to_string(),
            suggested_params: None,
        };
        assert!(output.suggested_params.is_none());
        assert!(!output.system_prompt.is_empty());
    }

    #[test]
    fn agent_output_with_params() {
        let params = GenerationParams {
            model: Some("gpt-4".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(2048),
        };
        let output = AgentOutput {
            system_prompt: "You are a QA engineer.".to_string(),
            user_context: "Write tests for this.".to_string(),
            suggested_params: Some(params),
        };
        assert!(output.suggested_params.is_some());
        let p = output.suggested_params.unwrap();
        assert_eq!(p.model.as_deref(), Some("gpt-4"));
        assert!((p.temperature.unwrap() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn agent_output_fields_are_owned_strings() {
        let s1 = String::from("prompt");
        let s2 = String::from("context");
        let output = AgentOutput {
            system_prompt: s1.clone(),
            user_context: s2.clone(),
            suggested_params: None,
        };
        let _ = output.system_prompt.clone();
        let _ = output.user_context.clone();
    }
}
