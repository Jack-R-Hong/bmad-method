// crates/bmad-types/src/metadata.rs

/// Static metadata for a BMAD agent, embedded at compile time.
/// All fields use `&'static str` to avoid heap allocation for constant data.
#[derive(Debug, Clone, Copy)]
pub struct AgentMetadata {
    /// Short programmatic name, lowercase hyphen-separated (e.g., "architect")
    pub name: &'static str,
    /// Human-readable display name (e.g., "Winston the Architect")
    pub display_name: &'static str,
    /// One-line description of the agent's purpose
    pub description: &'static str,
    /// Pulse executor name in `bmad/{id}` format (e.g., "bmad/architect")
    pub executor_name: &'static str,
    /// List of capability tags for discovery
    pub capabilities: &'static [&'static str],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_metadata_construction() {
        let meta = AgentMetadata {
            name: "architect",
            display_name: "Winston the Architect",
            description: "Architecture review and design guidance",
            executor_name: "bmad/architect",
            capabilities: &["architecture", "design", "review"],
        };
        assert_eq!(meta.name, "architect");
        assert_eq!(meta.executor_name, "bmad/architect");
        assert_eq!(meta.capabilities.len(), 3);
    }

    #[test]
    fn capabilities_is_static_slice() {
        let caps: &'static [&'static str] = &["planning", "analysis"];
        let meta = AgentMetadata {
            name: "pm",
            display_name: "John the PM",
            description: "Product management tasks",
            executor_name: "bmad/pm",
            capabilities: caps,
        };
        assert!(meta.capabilities.contains(&"planning"));
    }
}
