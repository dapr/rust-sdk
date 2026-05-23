use std::time::Duration;

use dapr::workflow::{
    ActivityContext, ActivityContextExt, RegistryExt, Result, ScheduleOptions, WorkflowClient,
    WorkflowContext, WorkflowContextExt, WorkflowError, when_all,
};

async fn batch_processing_workflow(ctx: WorkflowContext) -> Result<Option<String>> {
    let batch_size: i32 = ctx.get_input_typed()?;

    let work_batch: Vec<i32> = ctx.call_activity_typed("GetWorkBatch", batch_size).await?;

    let parallel_tasks = work_batch
        .into_iter()
        .map(|work_item| ctx.call_activity("ProcessWorkItem", work_item))
        .collect();

    let outputs = when_all(parallel_tasks)
        .await?
        .into_iter()
        .map(|output| match output {
            Some(value) => serde_json::from_str::<i32>(&value).map_err(WorkflowError::from),
            None => Ok(0),
        })
        .try_fold(0, |total, output| output.map(|value| total + value))?;

    let _: i32 = ctx.call_activity_typed("ProcessResults", outputs).await?;

    Ok(Some(serde_json::to_string(&0)?))
}

async fn get_work_batch(ctx: ActivityContext, input: Option<String>) -> Result<Option<String>> {
    let batch_size: i32 = ActivityContextExt::get_input(&ctx, input.as_deref())?;
    let batch = (0..batch_size).collect::<Vec<_>>();

    Ok(Some(serde_json::to_string(&batch)?))
}

async fn process_work_item(ctx: ActivityContext, input: Option<String>) -> Result<Option<String>> {
    let work_item: i32 = ActivityContextExt::get_input(&ctx, input.as_deref())?;

    println!("Processing work item: {work_item}");
    tokio::time::sleep(Duration::from_secs(5)).await;
    let result = work_item * 2;
    println!("Work item {work_item} processed. Result: {result}");

    Ok(Some(serde_json::to_string(&result)?))
}

async fn process_results(ctx: ActivityContext, input: Option<String>) -> Result<Option<String>> {
    let final_result: i32 = ActivityContextExt::get_input(&ctx, input.as_deref())?;

    println!("Final result: {final_result}");

    Ok(Some(serde_json::to_string(&final_result)?))
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init();

    let mut client = WorkflowClient::new().await?;
    client
        .registry_mut()
        .add_workflow("BatchProcessingWorkflow", batch_processing_workflow);
    client
        .registry_mut()
        .add_activity("GetWorkBatch", get_work_batch);
    client
        .registry_mut()
        .add_activity("ProcessWorkItem", process_work_item);
    client
        .registry_mut()
        .add_activity("ProcessResults", process_results);
    println!("Workflow(s) and activities registered.");
    println!("Worker initialized");

    let worker = client.start_worker().await?;

    let id = client
        .schedule_workflow(
            "BatchProcessingWorkflow",
            ScheduleOptions::new().with_input(10),
        )
        .await?;

    let metadata = client.wait_for_workflow_completion(&id).await?;
    metadata.raise_if_failed()?;
    println!(
        "workflow status: {}",
        format!("{:?}", metadata.runtime_status).to_uppercase()
    );

    client.terminate_workflow(&id).await?;
    println!("workflow terminated");

    client.purge_workflow_state(&id).await?;
    println!("workflow purged");

    worker.shutdown().await?;

    Ok(())
}
