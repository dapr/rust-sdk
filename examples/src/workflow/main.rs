use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

use dapr::workflow::{
    ActivityContext, ActivityContextExt, ActivityOptions, EventOptions, FetchOptions, RegistryExt,
    RetryPolicy, RuntimeStatus, ScheduleOptions, WorkflowClient, WorkflowContext,
    WorkflowContextExt, WorkflowError,
};

static STAGE: AtomicI32 = AtomicI32::new(0);
static FAIL_ACTIVITY_TRIES: AtomicI32 = AtomicI32::new(0);

const INSTANCE_ID: &str = "a7a4168d-3a1c-41da-8a4f-e7f6d9c718d9";
const TEST_WORKFLOW: &str = "TestWorkflow";
const TEST_ACTIVITY: &str = "TestActivity";
const FAIL_ACTIVITY: &str = "FailActivity";
const TEST_EVENT: &str = "testEvent";

async fn test_workflow(ctx: WorkflowContext) -> dapr::workflow::Result<Option<String>> {
    let input: i32 = ctx.get_input_typed()?;

    let _: String = ctx.call_activity_typed(TEST_ACTIVITY, input).await?;

    let event_payload: String = ctx
        .wait_for_external_event_typed(TEST_EVENT, Some(Duration::from_secs(60)))
        .await?;
    if !ctx.is_replaying() {
        println!("workflow received event payload: {event_payload}");
    }

    let output: String = ctx.call_activity_typed(TEST_ACTIVITY, input).await?;

    let retry_policy = RetryPolicy::new(3, Duration::from_millis(100))
        .with_backoff_coefficient(2.0)
        .with_max_retry_interval(Duration::from_secs(1));
    let fail_result = ctx
        .call_activity_with_options_typed::<String>(
            FAIL_ACTIVITY,
            ActivityOptions::new().with_retry_policy(retry_policy),
        )
        .await;
    if fail_result.is_ok() {
        return Err(WorkflowError::Other(
            "unexpected no error executing fail activity".to_string(),
        ));
    }

    Ok(Some(serde_json::to_string(&output)?))
}

async fn test_activity(
    ctx: ActivityContext,
    input: Option<String>,
) -> dapr::workflow::Result<Option<String>> {
    let input: i32 = ctx.get_input(input.as_deref())?;
    let stage = STAGE.fetch_add(input, Ordering::SeqCst) + input;
    Ok(Some(serde_json::to_string(&format!("Stage: {stage}"))?))
}

async fn fail_activity(
    _ctx: ActivityContext,
    _input: Option<String>,
) -> dapr::workflow::Result<Option<String>> {
    FAIL_ACTIVITY_TRIES.fetch_add(1, Ordering::SeqCst);
    Err(WorkflowError::Other("dummy activity error".to_string()))
}

#[tokio::main]
async fn main() -> dapr::workflow::Result<()> {
    env_logger::init();

    let mut client = WorkflowClient::new().await?;

    client
        .registry_mut()
        .add_workflow(TEST_WORKFLOW, test_workflow);
    println!("TestWorkflow registered");

    client
        .registry_mut()
        .add_activity(TEST_ACTIVITY, test_activity);
    println!("TestActivity registered");

    client
        .registry_mut()
        .add_activity(FAIL_ACTIVITY, fail_activity);
    println!("FailActivity registered");
    println!("Worker initialized");

    let worker = client.start_worker().await?;
    println!("runner started");

    let instance_id = client
        .schedule_workflow(
            TEST_WORKFLOW,
            ScheduleOptions::new()
                .with_instance_id(INSTANCE_ID)
                .with_input(1),
        )
        .await?;
    println!("workflow started with id: {instance_id}");

    client.suspend_workflow(&instance_id, "").await?;
    let metadata = client
        .fetch_workflow_metadata(&instance_id, FetchOptions::new().with_fetch_payloads(true))
        .await?;
    assert_eq!(metadata.runtime_status, RuntimeStatus::Suspended);
    println!("workflow paused");

    client.resume_workflow(&instance_id, "").await?;
    let metadata = client
        .fetch_workflow_metadata(&instance_id, FetchOptions::new().with_fetch_payloads(true))
        .await?;
    assert_eq!(metadata.runtime_status, RuntimeStatus::Running);
    println!("workflow resumed");
    println!("stage: {}", STAGE.load(Ordering::SeqCst));

    client
        .raise_event(
            &instance_id,
            TEST_EVENT,
            EventOptions::new().with_payload("testData"),
        )
        .await?;
    println!("workflow event raised");

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("stage: {}", STAGE.load(Ordering::SeqCst));

    client
        .wait_for_workflow_completion_with_options(
            &instance_id,
            FetchOptions::new().with_fetch_payloads(true),
            Some(Duration::from_secs(5)),
        )
        .await?;
    println!(
        "fail activity executions: {}",
        FAIL_ACTIVITY_TRIES.load(Ordering::SeqCst)
    );

    let metadata = client
        .fetch_workflow_metadata(&instance_id, FetchOptions::new().with_fetch_payloads(true))
        .await?;
    println!("workflow status: {:?}", metadata.runtime_status);

    client.purge_workflow_state(&instance_id).await?;
    if client
        .fetch_workflow_metadata(&instance_id, FetchOptions::new().with_fetch_payloads(true))
        .await
        .is_ok()
    {
        return Err(WorkflowError::Other(
            "workflow metadata still exists after purge".to_string(),
        ));
    }
    println!("workflow purged");
    println!("stage: {}", STAGE.load(Ordering::SeqCst));

    let id = client
        .schedule_workflow(
            TEST_WORKFLOW,
            ScheduleOptions::new()
                .with_instance_id(INSTANCE_ID)
                .with_input(1),
        )
        .await?;
    println!("workflow started with id: {id}");

    let metadata = client.wait_for_workflow_start(&id).await?;
    println!("workflow status: {:?}", metadata.runtime_status);

    client.terminate_workflow(&id).await?;
    println!("workflow terminated");

    client.purge_workflow_state(&id).await?;
    println!("workflow purged");

    worker.shutdown().await?;
    println!("workflow worker successfully shutdown");

    Ok(())
}
