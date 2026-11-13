use std::time::Duration;

use dapr::workflow::{
    ActivityContext, ActivityContextExt, CreateWorkflowScheduleOptions, FetchOptions,
    ListInstanceIdsOptions, ListWorkflowSchedulesOptions, RegistryExt, WorkflowClient,
    WorkflowContext, WorkflowContextExt, WorkflowError, WorkflowScheduleState,
};

const GREETING_WORKFLOW: &str = "GreetingWorkflow";
const GREET_ACTIVITY: &str = "Greet";
const SCHEDULE: &str = "greetings";

async fn greeting_workflow(ctx: WorkflowContext) -> dapr::workflow::Result<Option<String>> {
    let name: String = ctx.get_input_typed()?;
    let greeting: String = ctx.call_activity_typed(GREET_ACTIVITY, name).await?;
    Ok(Some(serde_json::to_string(&greeting)?))
}

async fn greet(
    ctx: ActivityContext,
    input: Option<String>,
) -> dapr::workflow::Result<Option<String>> {
    let name: String = ctx.get_input(input.as_deref())?;
    Ok(Some(serde_json::to_string(&format!("Hello {name}!"))?))
}

#[tokio::main]
async fn main() -> dapr::workflow::Result<()> {
    env_logger::init();

    let mut client = WorkflowClient::new().await?;
    client
        .registry_mut()
        .add_workflow(GREETING_WORKFLOW, greeting_workflow);
    client.registry_mut().add_activity(GREET_ACTIVITY, greet);
    println!("GreetingWorkflow registered");

    let worker = client.start_worker().await?;
    println!("Worker started");

    // A manual trigger counts as a run, so the schedule completes after the
    // triggered run and one scheduled run.
    client
        .create_workflow_schedule(
            SCHEDULE,
            GREETING_WORKFLOW,
            "@every 1s",
            CreateWorkflowScheduleOptions::new()
                .with_input("Dapr")
                .with_repeats(2)
                .with_overwrite(),
        )
        .await?;
    println!("schedule created");

    let run_id = client.trigger_workflow_schedule(SCHEDULE).await?;
    println!("manual run triggered: {}", run_id.ends_with("_manual"));

    let status = loop {
        let schedule = client.get_workflow_schedule(SCHEDULE).await?;
        if let Some(status) = schedule
            .status
            .filter(|s| s.state == WorkflowScheduleState::Completed)
        {
            break status;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    };
    println!("schedule completed after {} runs", status.run_count);

    let page = client
        .list_workflow_schedules(ListWorkflowSchedulesOptions::new())
        .await?;
    println!(
        "schedule listed: {}",
        page.schedules.iter().any(|s| s.name == SCHEDULE)
    );

    let run_ids = client
        .list_instance_ids(ListInstanceIdsOptions::new().with_instance_id_prefix("greetings_"))
        .await?
        .instance_ids;
    for id in &run_ids {
        let metadata = client
            .wait_for_workflow_completion_with_options(
                id,
                FetchOptions::new().with_fetch_payloads(true),
                Some(Duration::from_secs(30)),
            )
            .await?;
        println!(
            "run output: {}",
            metadata.serialized_output.unwrap_or_default()
        );
    }
    println!("runs completed: {}", run_ids.len());

    client.delete_workflow_schedule(SCHEDULE).await?;
    if client.get_workflow_schedule(SCHEDULE).await.is_ok() {
        return Err(WorkflowError::Other(
            "schedule still exists after delete".to_string(),
        ));
    }
    println!("schedule deleted");

    for id in &run_ids {
        client.purge_workflow_state(id).await?;
    }

    worker.shutdown().await?;
    println!("workflow worker successfully shutdown");

    Ok(())
}
