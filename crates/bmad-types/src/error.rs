// crates/bmad-types/src/error.rs

/// Typed errors for the bmad-method plugin.
/// Uses thiserror for stable, typed error interface at the plugin boundary.
/// Message format: lowercase, no trailing punctuation.
#[derive(thiserror::Error, Debug)]
pub enum BmadError {
    /// Returned when an executor name is not found in the registry
    #[error("agent '{0}' not found")]
    AgentNotFound(String),

    /// Returned when task input fails validation (e.g., empty input)
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Returned when agent execution encounters an internal failure
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_not_found_message() {
        let err = BmadError::AgentNotFound("architect".to_string());
        assert_eq!(err.to_string(), "agent 'architect' not found");
    }

    #[test]
    fn invalid_input_message() {
        let err = BmadError::InvalidInput("input cannot be empty".to_string());
        assert_eq!(err.to_string(), "invalid input: input cannot be empty");
    }

    #[test]
    fn execution_failed_message() {
        let err = BmadError::ExecutionFailed("timeout after 30s".to_string());
        assert_eq!(err.to_string(), "execution failed: timeout after 30s");
    }

    #[test]
    fn error_messages_are_lowercase() {
        let errors = vec![
            BmadError::AgentNotFound("test".to_string()),
            BmadError::InvalidInput("reason".to_string()),
            BmadError::ExecutionFailed("cause".to_string()),
        ];
        for err in errors {
            let msg = err.to_string();
            assert_eq!(
                msg,
                msg.to_lowercase(),
                "Error message must be lowercase: {}",
                msg
            );
            assert!(
                !msg.ends_with('.'),
                "Error message must not end with punctuation: {}",
                msg
            );
        }
    }
}
