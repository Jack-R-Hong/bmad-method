// crates/bmad-types/src/lib.rs
//! Shared type definitions for the bmad-method Pulse plugin.
//! These types are used by both bmad-converter (build-time) and bmad-plugin (runtime).
//! RULE: Never duplicate these types in other crates.

pub mod config;
pub mod error;
pub mod metadata;
pub mod output;
pub mod verification;

pub use config::BmadConfig;
pub use error::BmadError;
pub use metadata::AgentMetadata;
pub use output::{AgentOutput, GenerationParams, SuggestedConfig};
pub use verification::VerificationResult;
