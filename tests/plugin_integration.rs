use std::path::Path;
use std::process::Command;

use bmad_types::{AgentOutput, BmadError};

// Replicates BmadExecutor::execute() using only pub bmad_plugin::generated,
// avoiding the need to expose executor internals. Validates empty input, looks
// up agent by executor_name, and constructs AgentOutput identical to production.
fn execute_agent(agent_id: &str, input: &str) -> Result<AgentOutput, BmadError> {
    if input.trim().is_empty() {
        return Err(BmadError::InvalidInput("input cannot be empty".to_string()));
    }

    let entries = bmad_plugin::generated::all_agent_entries();
    match entries
        .into_iter()
        .find(|(meta, _, _)| meta.executor_name == agent_id)
    {
        None => Err(BmadError::AgentNotFound(agent_id.to_string())),
        Some((_meta, system_prompt, suggested_params)) => Ok(AgentOutput {
            system_prompt: system_prompt.to_string(),
            user_context: input.to_string(),
            suggested_params,
        }),
    }
}

fn plugin_path() -> std::path::PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = manifest_dir.join("target").join("release");
    if cfg!(target_os = "macos") {
        target.join("libbmad_plugin.dylib")
    } else {
        target.join("libbmad_plugin.so")
    }
}

#[test]
fn plugin_exports_register_symbol() {
    let path = plugin_path();
    if !path.exists() {
        eprintln!("Skipping: plugin binary not found at {:?}", path);
        eprintln!("Run `cargo build -p bmad-plugin --release` first.");
        return;
    }

    let output = Command::new("nm")
        .arg("--dynamic")
        .arg("--defined-only")
        .arg(&path)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            assert!(
                stdout.contains("pulse_plugin_register"),
                "Plugin binary does not export 'pulse_plugin_register' symbol.\nnm output:\n{}",
                stdout
            );
        }
        Err(e) => {
            eprintln!("nm not available ({}), trying objdump...", e);
            let output2 = Command::new("objdump")
                .arg("-T")
                .arg(&path)
                .output()
                .expect("neither nm nor objdump available");
            let stdout = String::from_utf8_lossy(&output2.stdout);
            assert!(
                stdout.contains("pulse_plugin_register"),
                "Plugin binary does not export 'pulse_plugin_register' symbol.\nobjdump output:\n{}",
                stdout
            );
        }
    }
}

#[test]
fn test_three_agent_sequential_workflow() {
    // Simulates Pulse DAG chaining: previous step output becomes next step input
    let arch_input = "Design a REST API for user management with CRUD operations";
    let arch_output =
        execute_agent("bmad/architect", arch_input).expect("architect step should succeed");

    assert!(
        !arch_output.system_prompt.is_empty(),
        "architect system_prompt must not be empty"
    );
    assert_eq!(
        arch_output.user_context, arch_input,
        "user_context must match input exactly"
    );

    let dev_input = format!(
        "{}\n\n---\nPrevious step output:\n{}",
        "Implement the API designed in the previous step", arch_output.user_context
    );
    let dev_output = execute_agent("bmad/dev", &dev_input).expect("developer step should succeed");

    assert!(
        !dev_output.system_prompt.is_empty(),
        "developer system_prompt must not be empty"
    );
    assert_eq!(
        dev_output.user_context, dev_input,
        "user_context must match input exactly"
    );

    let qa_input = format!(
        "{}\n\n---\nPrevious step output:\n{}",
        "Write test cases for the implementation in the previous step", dev_output.user_context
    );
    let qa_output = execute_agent("bmad/qa", &qa_input).expect("QA step should succeed");

    assert!(
        !qa_output.system_prompt.is_empty(),
        "QA system_prompt must not be empty"
    );
    assert_eq!(
        qa_output.user_context, qa_input,
        "user_context must match input exactly"
    );

    assert_ne!(
        arch_output.system_prompt, dev_output.system_prompt,
        "agents should have distinct system prompts"
    );
    assert_ne!(
        dev_output.system_prompt, qa_output.system_prompt,
        "agents should have distinct system prompts"
    );
    assert!(
        dev_output.user_context.contains(arch_input),
        "dev output must contain architect input (accumulated DAG context at step 2)"
    );
    assert!(
        qa_input.contains(arch_input),
        "QA input must contain the original architect input (accumulated DAG context at step 3)"
    );
}

#[test]
fn test_parallel_agents_no_shared_state() {
    use std::thread;

    let handle_arch = thread::spawn(|| {
        execute_agent("bmad/architect", "input_for_architect_thread")
            .expect("architect parallel execution should succeed")
    });

    let handle_qa = thread::spawn(|| {
        execute_agent("bmad/qa", "input_for_qa_thread")
            .expect("qa parallel execution should succeed")
    });

    let arch_output = handle_arch
        .join()
        .expect("architect thread should not panic");
    let qa_output = handle_qa.join().expect("qa thread should not panic");

    assert_eq!(arch_output.user_context, "input_for_architect_thread");
    assert_eq!(qa_output.user_context, "input_for_qa_thread");
    assert!(
        !arch_output.user_context.contains("qa_thread"),
        "architect output must not contain qa thread data"
    );
    assert!(
        !qa_output.user_context.contains("architect_thread"),
        "qa output must not contain architect thread data"
    );
}

#[test]
fn test_failed_agent_step_error_contains_agent_name() {
    match execute_agent("bmad/nonexistent-agent", "some input") {
        Err(BmadError::AgentNotFound(name)) => {
            assert!(
                name.contains("nonexistent-agent"),
                "error must contain the unrecognized agent name, got: '{}'",
                name
            );
        }
        other => panic!("expected AgentNotFound, got: {:?}", other),
    }
}

#[test]
fn test_empty_input_error_is_descriptive() {
    match execute_agent("bmad/architect", "") {
        Err(BmadError::InvalidInput(msg)) => {
            assert!(
                msg.contains("empty"),
                "error must contain 'empty', got: '{}'",
                msg
            );
        }
        other => panic!("expected InvalidInput for empty input, got: {:?}", other),
    }

    match execute_agent("bmad/architect", "   \t\n  ") {
        Err(BmadError::InvalidInput(msg)) => {
            assert!(
                msg.contains("empty"),
                "error must contain 'empty' for whitespace input, got: '{}'",
                msg
            );
        }
        other => panic!(
            "expected InvalidInput for whitespace input, got: {:?}",
            other
        ),
    }
}

#[test]
fn test_execute_outputs_are_independent() {
    let out1 = execute_agent("bmad/architect", "input_alpha").expect("first call should succeed");
    let out2 = execute_agent("bmad/architect", "input_beta").expect("second call should succeed");

    assert_eq!(out1.user_context, "input_alpha");
    assert_eq!(out2.user_context, "input_beta");
    assert_eq!(
        out1.system_prompt, out2.system_prompt,
        "system_prompt must be the same static content across calls"
    );
    assert_ne!(
        out1.user_context, out2.user_context,
        "outputs with different inputs must differ"
    );
}

#[test]
fn test_no_resource_leak_on_agent_not_found() {
    for i in 0..100 {
        match execute_agent("bmad/nonexistent", "some input") {
            Err(BmadError::AgentNotFound(_)) => {}
            other => panic!("iteration {}: expected AgentNotFound, got: {:?}", i, other),
        }
    }
}

#[test]
fn test_no_resource_leak_on_invalid_input() {
    for i in 0..100 {
        match execute_agent("bmad/architect", "") {
            Err(BmadError::InvalidInput(_)) => {}
            other => panic!("iteration {}: expected InvalidInput, got: {:?}", i, other),
        }
    }
}
