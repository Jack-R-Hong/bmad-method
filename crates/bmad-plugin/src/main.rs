#![forbid(unsafe_code)]

use bmad_plugin::BmadMethodPlugin;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let plugin = BmadMethodPlugin;
    pulse_plugin_sdk::dev_adapter::DevAdapter::new(plugin).serve_stdio();
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "step-executor-plugin",
    path: "wit/",
});

#[cfg(target_arch = "wasm32")]
struct _WitGuest;

#[cfg(target_arch = "wasm32")]
impl exports::pulse::plugin::plugin_lifecycle::Guest for _WitGuest {
    fn get_info() -> exports::pulse::plugin::plugin_lifecycle::PluginInfo {
        use pulse_plugin_sdk::wit_traits::PluginLifecycle as _;
        let p = BmadMethodPlugin::default();
        let info = p.get_info();
        exports::pulse::plugin::plugin_lifecycle::PluginInfo {
            name: info.name,
            version: info.version,
            description: info.description,
            dependencies: info
                .dependencies
                .into_iter()
                .map(
                    |d| exports::pulse::plugin::plugin_lifecycle::PluginDependency {
                        name: d.name,
                        version_req: d.version_req,
                        optional: d.optional,
                    },
                )
                .collect(),
        }
    }

    fn health_check() -> bool {
        use pulse_plugin_sdk::wit_traits::PluginLifecycle as _;
        let p = BmadMethodPlugin::default();
        p.health_check()
    }
}

#[cfg(target_arch = "wasm32")]
impl exports::pulse::plugin::step_executor::Guest for _WitGuest {
    fn execute(
        task: exports::pulse::plugin::step_executor::TaskInput,
        config: exports::pulse::plugin::step_executor::StepConfig,
    ) -> Result<exports::pulse::plugin::step_executor::StepResult, String> {
        use pulse_plugin_sdk::wit_traits::StepExecutorPlugin as _;
        use pulse_plugin_sdk::wit_types::{StepConfig as SdkStepConfig, TaskInput as SdkTaskInput};
        let p = BmadMethodPlugin::default();
        let sdk_task = SdkTaskInput {
            task_id: task.task_id,
            description: task.description,
            input: task.input,
            metadata: task.metadata,
        };
        let sdk_config = SdkStepConfig {
            step_id: config.step_id,
            step_type: config.step_type,
            timeout_secs: config.timeout_secs,
        };
        p.execute(sdk_task, sdk_config)
            .map(|r| exports::pulse::plugin::step_executor::StepResult {
                step_id: r.step_id,
                status: r.status,
                content: r.content,
                execution_time_ms: r.execution_time_ms,
            })
            .map_err(|e| e.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
export!(_WitGuest);
