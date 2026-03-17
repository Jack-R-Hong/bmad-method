// crates/bmad-plugin/src/executor.rs
//
// STUB INTERFACE (Story 2.1) — see docs/pulse-api-contract.md for verified real API
// ==================================================================================
// This file implements the BMAD-specific STUB TaskExecutor trait defined in
// pulse_api_stub.rs.  The stub is intentionally simplified for self-contained
// operation without a live Pulse binary.
//
// Stub trait signature (pulse_api_stub.rs):
//   pub trait TaskExecutor: Send + Sync {
//       fn executor_name(&self) -> &str;
//       fn execute(&self, input: &str) -> Result<AgentOutput, BmadError>;
//   }
//
// The REAL plugin-api trait was verified in Story 3.1 and differs significantly.
// See docs/pulse-api-contract.md (Architecture Assumptions vs Reality table) for
// the full comparison. Key differences are annotated with RECONCILED comments below.
//
// Activate real crate via `pulse-api` feature flag when available.

use bmad_types::{AgentMetadata, AgentOutput, BmadError, GenerationParams};

#[cfg(not(feature = "pulse-api"))]
use crate::pulse_api_stub::TaskExecutor;
#[cfg(feature = "pulse-api")]
use pulse_api::TaskExecutor;

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
}

impl TaskExecutor for BmadExecutor {
    // RECONCILED: stub uses executor_name(); real plugin-api uses name() + version().
    // When pulse-api feature is enabled, rename to name() and add version() returning
    // the crate version string. See docs/pulse-api-contract.md for full details.
    fn executor_name(&self) -> &str {
        self.metadata.executor_name
    }

    // RECONCILED: stub uses fn execute(&str) -> Result<AgentOutput, BmadError> (sync).
    // Real plugin-api uses async fn execute(&Task, &StepConfig) -> PluginResult<StepOutput>.
    // When pulse-api feature is enabled: add async, accept Task+StepConfig, return StepOutput.
    fn execute(&self, input: &str) -> Result<AgentOutput, BmadError> {
        // No blocking I/O — all data is statically embedded (NFR2: <500ms overhead requirement)
        if input.trim().is_empty() {
            return Err(BmadError::InvalidInput("input cannot be empty".to_string()));
        }

        Ok(AgentOutput {
            system_prompt: self.system_prompt.to_string(),
            user_context: input.to_string(),
            suggested_params: self.suggested_params.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated;

    static TEST_META: AgentMetadata = AgentMetadata {
        id: "test-agent",
        name: "test-agent",
        display_name: "Test Agent",
        description: "A test agent for unit testing",
        executor_name: "bmad/test-agent",
        capabilities: &["testing"],
    };

    const TEST_SYSTEM_PROMPT: &str = "You are a test agent. Be thorough and precise.";

    #[test]
    fn executor_returns_output_for_valid_input() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let result = exec.execute("Review this design.");
        assert!(result.is_ok());
        let Ok(output) = result else {
            panic!("executor returned unexpected error")
        };
        assert!(!output.system_prompt.is_empty());
        assert_eq!(output.user_context, "Review this design.");
    }

    #[test]
    fn executor_returns_error_for_empty_input() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let result = exec.execute("   ");
        assert!(matches!(result, Err(BmadError::InvalidInput(_))));
    }

    #[test]
    fn executor_name_matches_metadata() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        assert_eq!(exec.executor_name(), "bmad/test-agent");
    }

    #[test]
    fn system_prompt_uses_constant_not_description() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let Ok(output) = exec.execute("do something") else {
            panic!("executor returned unexpected error")
        };
        assert_eq!(output.system_prompt, TEST_SYSTEM_PROMPT);
        assert_ne!(output.system_prompt, TEST_META.description);
    }

    #[test]
    fn valid_agent_dispatch_returns_ok_with_full_system_prompt() {
        let exec = BmadExecutor::for_agent(
            &generated::architect::ARCHITECT,
            generated::architect::SYSTEM_PROMPT,
            generated::architect::suggested_params(),
        );
        assert_eq!(exec.executor_name(), "bmad/architect");
        let result = exec.execute("review the service mesh architecture");
        assert!(result.is_ok(), "Expected Ok from architect executor");
        let Ok(out) = result else {
            panic!("executor returned unexpected error")
        };
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
        let err = exec.execute("").expect_err("empty input must return Err");
        match err {
            BmadError::InvalidInput(ref msg) => {
                assert_eq!(
                    msg, "input cannot be empty",
                    "AC3: exact error message must match"
                );
            }
            other => panic!("expected InvalidInput, got: {:?}", other),
        }
    }

    #[test]
    fn execute_whitespace_only_returns_invalid_input() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let err = exec
            .execute("   \t\n  ")
            .expect_err("whitespace-only input must return Err");
        match err {
            BmadError::InvalidInput(ref msg) => {
                assert_eq!(
                    msg, "input cannot be empty",
                    "AC3: whitespace-only must give same error message"
                );
            }
            other => panic!("expected InvalidInput, got: {:?}", other),
        }
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
        let Ok(output) = exec.execute(input) else {
            panic!("executor returned unexpected error")
        };
        assert_eq!(output.user_context, input);
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
            let Ok(output) = exec.execute("test input") else {
                panic!("unexpected error for agent {}", meta.name)
            };
            assert!(
                !output.system_prompt.is_empty(),
                "system_prompt must be non-empty for agent {}",
                meta.name
            );
        }
    }

    #[test]
    fn two_outputs_from_same_executor_are_independent() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let Ok(out1) = exec.execute("first input") else {
            panic!("unexpected error on first call")
        };
        let Ok(out2) = exec.execute("second input") else {
            panic!("unexpected error on second call")
        };
        assert_ne!(out1.user_context, out2.user_context);
        assert_eq!(out1.system_prompt, out2.system_prompt);
    }

    #[test]
    fn outputs_own_independent_strings() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let Ok(out1) = exec.execute("input alpha") else {
            panic!("unexpected error")
        };
        let Ok(out2) = exec.execute("input beta") else {
            panic!("unexpected error")
        };
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
        let Ok(output) = exec.execute("review this") else {
            panic!("unexpected error")
        };
        let p = output
            .suggested_params
            .expect("expected Some(GenerationParams)");
        assert!((p.temperature.unwrap() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn suggested_params_none_when_not_specified() {
        let exec = BmadExecutor::for_agent(&TEST_META, TEST_SYSTEM_PROMPT, None);
        let Ok(output) = exec.execute("review this") else {
            panic!("unexpected error")
        };
        assert!(output.suggested_params.is_none());
    }

    // ── Persona validation tests (Story 2.3 Task 6) ──────────────────────────

    #[test]
    fn architect_system_prompt_contains_persona_keywords() {
        let exec = BmadExecutor::for_agent(
            &generated::architect::ARCHITECT,
            generated::architect::SYSTEM_PROMPT,
            generated::architect::suggested_params(),
        );
        let Ok(output) = exec.execute("test input") else {
            panic!("architect executor returned unexpected error")
        };
        let prompt_lower = output.system_prompt.to_lowercase();
        assert!(
            prompt_lower.contains("architect") || prompt_lower.contains("winston"),
            "Architect SYSTEM_PROMPT must contain 'architect' or 'winston', got: {}",
            &output.system_prompt[..200.min(output.system_prompt.len())]
        );
    }

    #[test]
    fn developer_system_prompt_contains_persona_keywords() {
        let exec = BmadExecutor::for_agent(
            &generated::developer::DEVELOPER,
            generated::developer::SYSTEM_PROMPT,
            generated::developer::suggested_params(),
        );
        let Ok(output) = exec.execute("test input") else {
            panic!("developer executor returned unexpected error")
        };
        let prompt_lower = output.system_prompt.to_lowercase();
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
        let Ok(output) = exec.execute("test input") else {
            panic!("pm executor returned unexpected error")
        };
        let prompt_lower = output.system_prompt.to_lowercase();
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
        let Ok(output) = exec.execute("test input") else {
            panic!("qa executor returned unexpected error")
        };
        let prompt_lower = output.system_prompt.to_lowercase();
        let has_keyword = prompt_lower.contains("test")
            || prompt_lower.contains("quality")
            || prompt_lower.contains("verify")
            || prompt_lower.contains("ship");
        assert!(
            has_keyword,
            "QA SYSTEM_PROMPT must contain 'test', 'quality', 'verify', or 'ship'"
        );
    }
}
