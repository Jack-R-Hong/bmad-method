// crates/bmad-types/src/verification.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationResult {
    pub executor_name: String,
    pub passed: bool,
    /// `None` when `passed` is `true`; contains the error message when `passed` is `false`.
    pub failure_reason: Option<String>,
}
