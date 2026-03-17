//! BMAD-METHOD Pulse plugin entry point.
//! Exports the pulse_plugin_register C symbol for dynamic loading by Pulse.

pub(crate) mod executor;
pub mod generated;
pub mod registry;

#[cfg(feature = "pulse-api")]
use pulse_api::{PluginMetadata, PluginRegistration, PLUGIN_API_VERSION};
#[cfg(not(feature = "pulse-api"))]
mod pulse_api_stub;
#[cfg(not(feature = "pulse-api"))]
use pulse_api_stub::{PluginMetadata, PluginRegistration, PLUGIN_API_VERSION};

use bmad_types::BmadError;
use executor::BmadExecutor;

// Task 3 (Story 4.5): pulse_api_stub::TaskExecutor and PluginRegistration do NOT include
// a verify() method. Pulse performs plugin verification by loading the shared library and
// calling pulse_plugin_register() — if that returns a non-null pointer without panicking,
// the plugin is considered verified. The Rust-side verify_all_agents() in registry.rs is
// available for programmatic health checks but is not wired to a Pulse verify hook because
// the current stub API does not define one. When the real pulse-api crate is available
// (feature = "pulse-api"), re-check whether PluginRegistration exposes a verify callback.
#[no_mangle]
pub unsafe extern "C" fn pulse_plugin_register() -> *mut PluginRegistration {
    match try_register() {
        Ok(registration) => Box::into_raw(Box::new(registration)),
        Err(e) => {
            eprintln!("bmad-method: failed to initialize plugin: {}", e);
            std::ptr::null_mut()
        }
    }
}

fn try_register() -> Result<PluginRegistration, BmadError> {
    let metadata =
        PluginMetadata::new("bmad-method", env!("CARGO_PKG_VERSION"), PLUGIN_API_VERSION);

    if registry::list_agents().is_empty() {
        return Err(BmadError::InvalidInput("no agents registered".to_string()));
    }

    let entries = generated::all_agent_entries();

    let mut registration = PluginRegistration::new(metadata);

    for (meta, prompt, params) in entries {
        let executor = BmadExecutor::for_agent(meta, prompt, params);
        registration = registration.with_task_executor(Box::new(executor));
    }

    Ok(registration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_register_succeeds() {
        let result = try_register();
        assert!(result.is_ok(), "Registration failed: {:?}", result.err());
    }

    #[test]
    fn all_agents_returns_at_least_one() {
        let agents = generated::all_agents();
        assert!(!agents.is_empty(), "No agents registered");
    }

    #[test]
    fn all_executor_names_start_with_bmad_prefix() {
        let agents = generated::all_agents();
        for agent in agents {
            assert!(
                agent.executor_name.starts_with("bmad/"),
                "Executor name '{}' must start with 'bmad/'",
                agent.executor_name
            );
        }
    }
}
