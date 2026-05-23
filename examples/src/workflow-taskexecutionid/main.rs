use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use dapr::workflow::{
    ActivityContext, ActivityContextExt, ActivityOptions, RegistryExt, RetryPolicy,
    ScheduleOptions, WorkflowClient, WorkflowContext, WorkflowContextExt, WorkflowError,
};

static EXECUTION_COUNTS: OnceLock<Mutex<HashMap<String, i32>>> = OnceLock::new();

const TASK_EXECUTION_ID_WORKFLOW: &str = "TaskExecutionIdWorkflow";
const RETRY_N: &str = "RetryN";

async fn task_execution_id_workflow(
    ctx: WorkflowContext,
) -> dapr::workflow::Result<Option<String>> {
    let retries: i32 = ctx.get_input_typed()?;

    let retry_policy = || {
        RetryPolicy::new(retries as u32, Duration::from_millis(100))
            .with_backoff_coefficient(2.0)
            .with_max_retry_interval(Duration::from_secs(1))
    };

    let _work_batch: Vec<i32> = ctx
        .call_activity_with_options_typed(
            RETRY_N,
            ActivityOptions::new()
                .with_retry_policy(retry_policy())
                .with_input(retries),
        )
        .await?;

    let _work_batch: Vec<i32> = ctx
        .call_activity_with_options_typed(
            RETRY_N,
            ActivityOptions::new()
                .with_retry_policy(retry_policy())
                .with_input(retries),
        )
        .await?;

    Ok(Some(serde_json::to_string(&0_i32)?))
}

async fn retry_n(
    ctx: ActivityContext,
    input: Option<String>,
) -> dapr::workflow::Result<Option<String>> {
    let retries: i32 = ctx.get_input(input.as_deref())?;
    let task_execution_id = ctx.task_execution_id().to_string();
    let count = {
        let execution_counts = EXECUTION_COUNTS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut execution_counts = execution_counts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let counter = execution_counts.entry(task_execution_id).or_insert(0);
        *counter += 1;
        *counter
    };

    println!("RetryN  {count}");

    if count < retries {
        return Err(WorkflowError::Other("failed".to_string()));
    }

    let work_batch: Vec<i32> = (1..=count).collect();
    Ok(Some(serde_json::to_string(&work_batch)?))
}

#[tokio::main]
async fn main() -> dapr::workflow::Result<()> {
    env_logger::init();

    let mut client = WorkflowClient::new().await?;

    client
        .registry_mut()
        .add_workflow(TASK_EXECUTION_ID_WORKFLOW, task_execution_id_workflow);
    client.registry_mut().add_activity(RETRY_N, retry_n);
    println!("Workflow(s) and activities registered.");
    println!("Worker initialized");

    let worker = client.start_worker().await?;

    let id = client
        .schedule_workflow(
            TASK_EXECUTION_ID_WORKFLOW,
            ScheduleOptions::new().with_input(5_i32),
        )
        .await?;

    let metadata = client.wait_for_workflow_completion(&id).await?;
    metadata.raise_if_failed()?;
    println!("workflow status: {:?}", metadata.runtime_status);

    client.terminate_workflow(&id).await?;
    println!("workflow terminated");

    client.purge_workflow_state(&id).await?;
    println!("workflow purged");

    worker.shutdown().await?;

    Ok(())
}
