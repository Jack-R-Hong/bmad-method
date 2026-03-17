// crates/bmad-plugin/src/pulse_api_stub.rs
// Stub types matching the expected Pulse Plugin API v0.1.x interface.

pub const PLUGIN_API_VERSION: u32 = 1;

pub trait TaskExecutor: Send + Sync {
    fn executor_name(&self) -> &str;
    fn execute(&self, input: &str) -> Result<bmad_types::AgentOutput, bmad_types::BmadError>;
}

pub struct PluginMetadata {
    pub name: &'static str,
    pub version: &'static str,
    pub api_version: u32,
}

impl PluginMetadata {
    pub fn new(name: &'static str, version: &'static str, api_version: u32) -> Self {
        Self {
            name,
            version,
            api_version,
        }
    }
}

pub struct PluginRegistration {
    pub metadata: PluginMetadata,
    pub executors: Vec<Box<dyn TaskExecutor>>,
}

impl PluginRegistration {
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            executors: Vec::new(),
        }
    }

    pub fn with_task_executor(mut self, executor: Box<dyn TaskExecutor>) -> Self {
        self.executors.push(executor);
        self
    }
}
