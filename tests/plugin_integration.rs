use bmad_types::BmadError;
use pulse_plugin_sdk::wit_types::{StepConfig, StepResult, TaskInput};

fn execute_agent(agent_id: &str, prompt: &str) -> Result<StepResult, BmadError> {
    if prompt.trim().is_empty() {
        return Err(BmadError::InvalidInput("input cannot be empty".to_string()));
    }

    let entries = bmad_plugin::generated::all_agent_entries();
    if entries
        .iter()
        .all(|(meta, _, _, _)| meta.executor_name != agent_id)
    {
        return Err(BmadError::AgentNotFound(agent_id.to_string()));
    }

    let input = serde_json::json!({
        "agent": agent_id,
        "prompt": prompt
    });

    let plugin = bmad_plugin::BmadMethodPlugin::default();
    let task = TaskInput::new("integration-test", prompt).with_input(input);
    let config = StepConfig::new("integration-step", "agent");

    use pulse_plugin_sdk::wit_traits::StepExecutorPlugin as _;
    plugin.execute(task, config).map_err(|e| {
        if e.code == "not_found" {
            BmadError::AgentNotFound(agent_id.to_string())
        } else {
            BmadError::InvalidInput(e.message)
        }
    })
}

fn parse_output(result: &StepResult) -> serde_json::Value {
    serde_json::from_str(result.content.as_deref().expect("content must be present")).unwrap()
}

#[test]
fn test_three_agent_sequential_workflow() {
    let arch_input = "Design a REST API for user management with CRUD operations";
    let arch_result =
        execute_agent("bmad/architect", arch_input).expect("architect step should succeed");
    let arch_out = parse_output(&arch_result);

    assert!(
        !arch_out["system_prompt"].as_str().unwrap_or("").is_empty(),
        "architect system_prompt must not be empty"
    );
    assert_eq!(
        arch_out["user_context"].as_str().unwrap(),
        arch_input,
        "user_context must match input exactly"
    );

    let dev_input = format!(
        "{}\n\n---\nPrevious step output:\n{}",
        "Implement the API designed in the previous step",
        arch_out["user_context"].as_str().unwrap()
    );
    let dev_result = execute_agent("bmad/dev", &dev_input).expect("developer step should succeed");
    let dev_out = parse_output(&dev_result);

    assert!(
        !dev_out["system_prompt"].as_str().unwrap_or("").is_empty(),
        "developer system_prompt must not be empty"
    );
    assert_eq!(
        dev_out["user_context"].as_str().unwrap(),
        dev_input,
        "user_context must match input exactly"
    );

    let qa_input = format!(
        "{}\n\n---\nPrevious step output:\n{}",
        "Write test cases for the implementation in the previous step",
        dev_out["user_context"].as_str().unwrap()
    );
    let qa_result = execute_agent("bmad/qa", &qa_input).expect("QA step should succeed");
    let qa_out = parse_output(&qa_result);

    assert!(
        !qa_out["system_prompt"].as_str().unwrap_or("").is_empty(),
        "QA system_prompt must not be empty"
    );
    assert_eq!(
        qa_out["user_context"].as_str().unwrap(),
        qa_input,
        "user_context must match input exactly"
    );

    assert_ne!(
        arch_out["system_prompt"].as_str().unwrap(),
        dev_out["system_prompt"].as_str().unwrap(),
        "agents should have distinct system prompts"
    );
    assert_ne!(
        dev_out["system_prompt"].as_str().unwrap(),
        qa_out["system_prompt"].as_str().unwrap(),
        "agents should have distinct system prompts"
    );
    assert!(
        dev_out["user_context"]
            .as_str()
            .unwrap()
            .contains(arch_input),
        "dev output must contain architect input (accumulated DAG context at step 2)"
    );
    assert!(
        qa_input.contains(arch_input),
        "QA input must contain the original architect input (accumulated DAG context at step 3)"
    );

    // Story 8.4: validate suggested_config in sequential workflow output
    assert!(arch_out["suggested_config"].is_object(), "architect must have suggested_config");
    assert_eq!(arch_out["suggested_config"]["model_tier"].as_str().unwrap(), "opus");
    assert_eq!(arch_out["suggested_config"]["permission_mode"].as_str().unwrap(), "plan");

    assert!(dev_out["suggested_config"].is_object(), "dev must have suggested_config");
    assert_eq!(dev_out["suggested_config"]["model_tier"].as_str().unwrap(), "sonnet");
    assert_eq!(dev_out["suggested_config"]["permission_mode"].as_str().unwrap(), "bypassPermissions");

    assert!(qa_out["suggested_config"].is_object(), "qa must have suggested_config");
    assert_eq!(qa_out["suggested_config"]["model_tier"].as_str().unwrap(), "sonnet");
    assert!(qa_out["suggested_config"]["max_turns"].as_u64().unwrap() > 0);

    // Story 9.1: schema_version present
    assert_eq!(arch_out["schema_version"].as_str().unwrap(), "1.1");
    assert_eq!(dev_out["schema_version"].as_str().unwrap(), "1.1");
    assert_eq!(qa_out["schema_version"].as_str().unwrap(), "1.1");

    // Story 9.2: capabilities in metadata
    assert!(arch_out["metadata"]["capabilities"].is_array());
    assert!(!arch_out["metadata"]["capabilities"].as_array().unwrap().is_empty());
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

    let arch_result = handle_arch
        .join()
        .expect("architect thread should not panic");
    let qa_result = handle_qa.join().expect("qa thread should not panic");

    let arch_out = parse_output(&arch_result);
    let qa_out = parse_output(&qa_result);

    assert_eq!(
        arch_out["user_context"].as_str().unwrap(),
        "input_for_architect_thread"
    );
    assert_eq!(
        qa_out["user_context"].as_str().unwrap(),
        "input_for_qa_thread"
    );
    assert!(
        !arch_out["user_context"]
            .as_str()
            .unwrap()
            .contains("qa_thread"),
        "architect output must not contain qa thread data"
    );
    assert!(
        !qa_out["user_context"]
            .as_str()
            .unwrap()
            .contains("architect_thread"),
        "qa output must not contain architect thread data"
    );

    // Story 8.4: validate suggested_config in parallel outputs
    assert!(arch_out["suggested_config"].is_object());
    assert!(qa_out["suggested_config"].is_object());
    assert_eq!(arch_out["suggested_config"]["model_tier"].as_str().unwrap(), "opus");
    assert_eq!(qa_out["suggested_config"]["model_tier"].as_str().unwrap(), "sonnet");
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
    let r1 = execute_agent("bmad/architect", "input_alpha").expect("first call should succeed");
    let r2 = execute_agent("bmad/architect", "input_beta").expect("second call should succeed");
    let out1 = parse_output(&r1);
    let out2 = parse_output(&r2);

    assert_eq!(out1["user_context"].as_str().unwrap(), "input_alpha");
    assert_eq!(out2["user_context"].as_str().unwrap(), "input_beta");
    assert_eq!(
        out1["system_prompt"].as_str().unwrap(),
        out2["system_prompt"].as_str().unwrap(),
        "system_prompt must be the same static content across calls"
    );
    assert_ne!(
        out1["user_context"].as_str().unwrap(),
        out2["user_context"].as_str().unwrap(),
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
