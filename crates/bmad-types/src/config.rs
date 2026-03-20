// crates/bmad-types/src/config.rs

use serde::Deserialize;

use crate::BmadError;

/// Step-level configuration for the bmad-method plugin.
/// `deny_unknown_fields` ensures YAML typos surface as errors instead of silently being ignored.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BmadConfig {
    pub agent_name: String,
    pub context: Option<String>,
}

impl BmadConfig {
    pub fn from_step_config(value: &str) -> Result<Self, BmadError> {
        serde_json::from_str(value).map_err(|e| {
            BmadError::InvalidInput(format!(
                "invalid step config: {e}; expected {{\"agent_name\": \"<name>\"}}"
            ))
        })
    }

    /// Returns the executor name in `bmad/<name>` format.
    /// Accepts both `"architect"` and `"bmad/architect"` as input.
    pub fn executor_name(&self) -> String {
        if self.agent_name.starts_with("bmad/") {
            self.agent_name.clone()
        } else {
            format!("bmad/{}", self.agent_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_field() {
        let cfg = BmadConfig::from_step_config(r#"{"agent_name": "architect"}"#).unwrap();
        assert_eq!(cfg.agent_name, "architect");
        assert!(cfg.context.is_none());
    }

    #[test]
    fn parses_with_optional_context() {
        let cfg = BmadConfig::from_step_config(
            r#"{"agent_name": "dev", "context": "focus on error handling"}"#,
        )
        .unwrap();
        assert_eq!(cfg.agent_name, "dev");
        assert_eq!(cfg.context.as_deref(), Some("focus on error handling"));
    }

    #[test]
    fn rejects_unknown_fields() {
        let result = BmadConfig::from_step_config(r#"{"agent_name": "qa", "typo": true}"#);
        assert!(
            matches!(result, Err(BmadError::InvalidInput(_))),
            "deny_unknown_fields must reject unknown keys"
        );
    }

    #[test]
    fn rejects_missing_agent_name() {
        let result = BmadConfig::from_step_config(r#"{"context": "some context"}"#);
        assert!(
            matches!(result, Err(BmadError::InvalidInput(_))),
            "agent_name is required"
        );
    }

    #[test]
    fn rejects_invalid_json() {
        let result = BmadConfig::from_step_config("not json");
        assert!(matches!(result, Err(BmadError::InvalidInput(_))));
    }

    #[test]
    fn executor_name_adds_prefix_when_missing() {
        let cfg = BmadConfig::from_step_config(r#"{"agent_name": "architect"}"#).unwrap();
        assert_eq!(cfg.executor_name(), "bmad/architect");
    }

    #[test]
    fn executor_name_preserves_existing_prefix() {
        let cfg = BmadConfig::from_step_config(r#"{"agent_name": "bmad/architect"}"#).unwrap();
        assert_eq!(cfg.executor_name(), "bmad/architect");
    }
}
