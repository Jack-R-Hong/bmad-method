# Pulse Plugin API Contract

**Verified:** 2026-03-17
**Source:** `/home/jack/Document/pulse/crates/plugin-api/src/` (local Pulse source tree)
**Story:** 3.1 — BMAD Frontmatter Schema and pulse-api Contract Verification

This document records the verified pulse `plugin-api` crate interface, the discrepancies between the architecture assumptions and the actual crate, and the resolution decisions that govern how this plugin is implemented.

---

## Trait Definition

The actual `TaskExecutor` trait from `crates/plugin-api/src/task_executor.rs`:

```rust
use crate::error::PluginResult;
use async_trait::async_trait;

#[async_trait]
pub trait TaskExecutor: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn execute(&self, task: &Task, config: &StepConfig) -> PluginResult<StepOutput>;
}
```

---

## Method Breakdown

| Method | Parameters | Ownership | Return Type |
|--------|-----------|-----------|-------------|
| `name` | `&self` | shared ref | `&str` |
| `version` | `&self` | shared ref | `&str` |
| `execute` | `&self`, `task: &Task`, `config: &StepConfig` | shared refs | `PluginResult<StepOutput>` (async) |

### Supporting Types

**`Task`** (input to execute):

```rust
pub struct Task {
    pub task_id: String,
    pub description: String,
    pub input: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}
```

**`StepConfig`** (execution configuration):

```rust
pub struct StepConfig {
    pub step_id: String,
    pub step_type: String,
    pub parameters: Option<serde_json::Value>,
    pub timeout_secs: Option<u64>,
    pub retry_count: u32,
}
```

**`StepOutput`** (result):

```rust
pub struct StepOutput {
    pub step_id: String,
    pub status: String,
    pub content: String,
    pub execution_time_ms: u64,
    pub metadata: Option<serde_json::Value>,
}
```

**`PluginResult<T>`**: Type alias for `Result<T, PluginError>`.

---

## Trait Bounds

- `Send`: Required — Pulse DAG executor dispatches tasks across threads
- `Sync`: Required — Multiple DAG paths may share a reference to the same executor concurrently
- `async fn execute`: The real API is fully async (uses `async-trait` crate for object safety)

---

## Registration API

**`PluginRegistration`** from `crates/plugin-api/src/registration.rs`:

```rust
pub struct PluginRegistration {
    pub metadata: PluginMetadata,
    pub quality_checks: Vec<Box<dyn QualityCheck>>,
    pub triage_engines: Vec<Box<dyn TriageEngine>>,
    pub routing_engines: Vec<Box<dyn RoutingEngine>>,
    pub task_executors: Vec<Box<dyn TaskExecutor>>,
    pub notify_channels: Vec<Box<dyn NotifyChannel>>,
    pub trigger_sources: Vec<Box<dyn TriggerSource>>,
    pub extension: Option<PluginExtension>,
}

impl PluginRegistration {
    pub fn new(metadata: PluginMetadata) -> Self { ... }
    pub fn with_task_executor(mut self, executor: Box<dyn TaskExecutor>) -> Self { ... }
    // also: with_quality_check, with_triage_engine, with_routing_engine,
    //       with_notify_channel, with_trigger_source, with_extension
}
```

Key difference from stub: the field is `task_executors` (not `executors`), and `PluginRegistration` supports multiple capability types beyond just task executors.

---

## Plugin Metadata

**`PluginMetadata`** from `crates/plugin-api/src/metadata.rs`:

```rust
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub api_version: String,       // semver string, e.g. "0.1.0"
    pub description: Option<String>,
    pub author: Option<String>,
}

impl PluginMetadata {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        api_version: impl Into<String>,
    ) -> Self { ... }

    pub fn with_description(mut self, description: impl Into<String>) -> Self { ... }
    pub fn with_author(mut self, author: impl Into<String>) -> Self { ... }
}
```

---

## API Version Constant

The real `plugin-api` crate **does** export version constants from `crates/plugin-api/src/lib.rs`:

```rust
pub const PLUGIN_API_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PLUGIN_API_VERSION_MAJOR: u32 = 0;
pub const PLUGIN_API_VERSION_MINOR: u32 = 1;
pub const PLUGIN_API_VERSION_PATCH: u32 = 0;
pub const MIN_COMPATIBLE_VERSION: &str = "0.1.0";
```

- **`PLUGIN_API_VERSION: &str`** — the crate's package version string (e.g., `"0.1.0"`). Always use this constant when constructing `PluginMetadata` rather than a hardcoded string literal — it stays current when the crate version bumps.
- **`PLUGIN_API_VERSION_MAJOR/MINOR/PATCH: u32`** — individual semver components for programmatic version comparisons.
- **`MIN_COMPATIBLE_VERSION: &str`** — the oldest API version that is backward-compatible with the current release.

Compatibility checking uses `ApiVersion` and `CompatibilityChecker`:

```rust
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
```

Compatibility rule: same `major` version = compatible. The `CompatibilityChecker::is_compatible()` method enforces this.

---

## Architecture Assumptions vs Reality

| Aspect | Architecture / Stub Assumed | Actual plugin-api | Resolution |
|--------|----------------------------|-------------------|------------|
| Method `executor_name()` | `fn executor_name(&self) -> &str` | `fn name(&self) -> &str` | **Stub kept with original name** — stub is intentionally BMAD-specific; it is not imported alongside the real crate. When `pulse-api` feature is enabled, the real trait is used. See `// RECONCILED` comment in `executor.rs`. |
| Method `version()` | Not present in stub | `fn version(&self) -> &str` | Stub omits `version()`. The real trait requires it. When toggling to real crate (`pulse-api` feature), `BmadExecutor` must implement `version()`. |
| `execute()` signature | `fn execute(&self, input: &str) -> Result<AgentOutput, BmadError>` | `async fn execute(&self, task: &Task, config: &StepConfig) -> PluginResult<StepOutput>` | **Significant difference.** Stub uses simplified synchronous interface with raw `&str` input. Real API is async and uses structured `Task`/`StepConfig` types. Stub approach is correct for current self-contained implementation; real-API integration deferred to feature-flag path. |
| `execute()` async | Synchronous | Async (requires `async-trait`) | Real API requires `async-trait` dependency. Not yet added — deferred to feature-flag integration. |
| `PluginMetadata` field types | `&'static str` for name/version, `u32` for api_version | `String` for all, semver `String` for api_version | Stub uses static strings for zero-allocation. Real API uses owned `String`. Resolved: stub fields stay as-is; real `PluginMetadata` constructed from owned strings at registration time. |
| `api_version` type | `u32` (integer `1`) | `String` (semver `"0.1.0"`) | Real API uses semver strings, not integers. |
| `PLUGIN_API_VERSION` constant | `pub const PLUGIN_API_VERSION: u32 = 1` | `pub const PLUGIN_API_VERSION: &str = env!("CARGO_PKG_VERSION")` plus `PLUGIN_API_VERSION_MAJOR/MINOR/PATCH: u32` and `MIN_COMPATIBLE_VERSION: &str` | Type mismatch: stub uses `u32`, real uses `&str`. The real crate exports the constant — use `plugin_api::PLUGIN_API_VERSION` when constructing `PluginMetadata.api_version` in the real integration. |
| `PluginRegistration.executors` | Field named `executors: Vec<Box<dyn TaskExecutor>>` | Field named `task_executors: Vec<Box<dyn TaskExecutor>>` | Stub field name differs. Registration builder method `with_task_executor()` is identical in both. |
| Registration scope | `PluginRegistration` holds only `executors` | `PluginRegistration` holds 7 capability types | Real API supports quality checks, triage, routing, notify, triggers. Plugin only registers task executors. |

---

## Verified Import Paths

When the `pulse-api` feature is enabled (future integration):

```rust
// In lib.rs / executor.rs
use plugin_api::task_executor::{TaskExecutor, Task, StepConfig, StepOutput};
use plugin_api::{PluginMetadata, PluginResult};
use plugin_api::registration::PluginRegistration;
use plugin_api::{PLUGIN_API_VERSION, MIN_COMPATIBLE_VERSION};
use async_trait::async_trait;
```

**`StepOutput` aliasing hazard:** `task_executor::StepOutput` is re-exported at the crate root as `ExecutorStepOutput` (not `StepOutput`) to avoid a naming conflict with `quality_check::StepOutput`. The module-path import above (`task_executor::StepOutput`) is safe and unambiguous. However, `use plugin_api::StepOutput` will fail to compile — there is no `plugin_api::StepOutput` at the crate root, only `plugin_api::ExecutorStepOutput`.

```rust
use plugin_api::ExecutorStepOutput;   // correct root-level import
use plugin_api::task_executor::StepOutput; // also correct (module-path)
// use plugin_api::StepOutput;         // DOES NOT EXIST — compile error
```

**Version constant usage:**

```rust
let metadata = PluginMetadata::new(
    "bmad-method",
    env!("CARGO_PKG_VERSION"),
    plugin_api::PLUGIN_API_VERSION,   // &str, e.g. "0.1.0"
);
```

Current (stub) import paths used without feature flag:

```rust
use crate::pulse_api_stub::TaskExecutor;
```

The conditional compilation in `executor.rs`:

```rust
#[cfg(not(feature = "pulse-api"))]
use crate::pulse_api_stub::TaskExecutor;
#[cfg(feature = "pulse-api")]
use pulse_api::TaskExecutor;
```

---

## Output Security Contract

The plugin passes `user_context` through verbatim. It does NOT sanitize, escape, or filter input content. Consumers that render `user_context` or `system_prompt` in HTML or other injection-sensitive contexts MUST sanitize the content before rendering.

**`suggested_config` values are advisory.** The workflow YAML takes precedence. Consumers MUST review `permission_mode` values before auto-applying, especially `bypassPermissions`.

If `prompt` is omitted from `BmadInput`, `task.description` is used as user context. A `tracing::warn!` is emitted when this fallback occurs.

**Input size limit:** Input is bounded to 128KB (131072 bytes). Inputs exceeding this limit are rejected with an `invalid_input` error.

---

## BmadExecutor Implementation Notes

The current `BmadExecutor` implements the **stub** `TaskExecutor` trait (not the real plugin-api trait). This is intentional — the plugin runs standalone without a live Pulse binary. Key characteristics:

- `executor_name()` returns `metadata.executor_name` (maps to the real `name()` method conceptually)
- `execute(&str)` accepts raw text input instead of structured `Task` + `StepConfig`
- Synchronous — no async runtime dependency
- Returns `AgentOutput` (BMAD type) not `StepOutput` (plugin-api type)

When the `pulse-api` feature flag is enabled, `BmadExecutor` must be updated to:
1. Implement `name()` and `version()` instead of `executor_name()`
2. Implement `async fn execute(&self, task: &Task, config: &StepConfig) -> PluginResult<StepOutput>`
3. Extract the text input from `task.input` or `task.description`
4. Wrap the BMAD `AgentOutput` content into a `StepOutput`
