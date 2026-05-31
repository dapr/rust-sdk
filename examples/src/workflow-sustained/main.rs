use std::env;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dapr::workflow::{
    ActivityContext, ActivityContextExt, FetchOptions, RegistryExt, Result, ScheduleOptions,
    WorkflowClient, WorkflowContext, WorkflowContextExt, WorkflowSchedulingClient,
};
use tokio::task::JoinSet;

const WORKFLOW_NAME: &str = "SustainedWorkflow";
const ACTIVITY_NAME: &str = "DoWork";
const DEFAULT_WORKFLOW_COUNT: usize = 100;
const MAX_WORKFLOW_COUNT: usize = 10_000;
const WORKFLOW_TIMEOUT: Duration = Duration::from_secs(120);

async fn sustained_workflow(ctx: WorkflowContext) -> Result<Option<String>> {
    let input: i32 = ctx.get_input_typed()?;
    let output: i32 = ctx.call_activity_typed(ACTIVITY_NAME, input).await?;

    Ok(Some(serde_json::to_string(&output)?))
}

async fn do_work(ctx: ActivityContext, input: Option<String>) -> Result<Option<String>> {
    let input: i32 = ctx.get_input(input.as_deref())?;
    let jitter_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::from(duration.subsec_nanos()) % 100)
        .unwrap_or(0);

    tokio::time::sleep(Duration::from_millis(50 + jitter_ms)).await;

    Ok(Some(serde_json::to_string(&(input * 2))?))
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init();

    let workflow_count = read_workflow_count();
    let concurrency = read_concurrency(workflow_count);

    eprintln!("Starting sustained workflow run: count={workflow_count}, concurrency={concurrency}");

    let mut worker_client = WorkflowClient::new().await?;
    worker_client
        .registry_mut()
        .add_workflow(WORKFLOW_NAME, sustained_workflow);
    worker_client
        .registry_mut()
        .add_activity(ACTIVITY_NAME, do_work);
    println!("Workflow(s) and activities registered.");
    println!("Worker initialized");

    let worker = worker_client.start_worker().await?;
    let scheduling_client = worker_client.scheduling_client();
    let started_at = Instant::now();

    let latencies = run_workflows(scheduling_client, workflow_count, concurrency).await;
    let elapsed = started_at.elapsed();

    print_summary(workflow_count, &latencies, elapsed);

    worker.shutdown().await?;

    Ok(())
}

fn read_workflow_count() -> usize {
    env::var("WORKFLOW_COUNT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(DEFAULT_WORKFLOW_COUNT)
        .clamp(1, MAX_WORKFLOW_COUNT)
}

fn read_concurrency(workflow_count: usize) -> usize {
    env::var("WORKFLOW_CONCURRENCY")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(workflow_count)
        .clamp(1, workflow_count)
}

async fn run_workflows(
    scheduling_client: WorkflowSchedulingClient,
    workflow_count: usize,
    concurrency: usize,
) -> Vec<Duration> {
    let mut tasks = JoinSet::new();
    let mut next_input = 0;
    let mut completed = 0;
    let mut failed = 0;
    let mut latencies = Vec::with_capacity(workflow_count);
    let progress_interval = (workflow_count / 10).max(1);

    while next_input < workflow_count && tasks.len() < concurrency {
        spawn_workflow(&mut tasks, scheduling_client.clone(), next_input as i32);
        next_input += 1;
    }

    while let Some(result) = tasks.join_next().await {
        completed += 1;
        match result {
            Ok(Ok(latency)) => latencies.push(latency),
            Ok(Err(error)) => {
                failed += 1;
                eprintln!("Workflow failed: {error}");
            }
            Err(error) => {
                failed += 1;
                eprintln!("Workflow task failed: {error}");
            }
        }

        if completed % progress_interval == 0 || completed == workflow_count {
            eprintln!(
                "Progress: {completed}/{workflow_count} completed ({} succeeded, {failed} failed)",
                latencies.len()
            );
        }

        if next_input < workflow_count {
            spawn_workflow(&mut tasks, scheduling_client.clone(), next_input as i32);
            next_input += 1;
        }
    }

    latencies
}

fn spawn_workflow(
    tasks: &mut JoinSet<std::result::Result<Duration, String>>,
    mut client: WorkflowSchedulingClient,
    input: i32,
) {
    tasks.spawn(async move {
        let started_at = Instant::now();
        let instance_id = client
            .schedule_workflow(WORKFLOW_NAME, ScheduleOptions::new().with_input(input))
            .await
            .map_err(|error| format!("schedule failed: {error}"))?;

        let completion = client
            .wait_for_workflow_completion_with_options(
                &instance_id,
                FetchOptions::new(),
                Some(WORKFLOW_TIMEOUT),
            )
            .await
            .map_err(|error| format!("wait failed for {instance_id}: {error}"))?;
        completion
            .raise_if_failed()
            .map_err(|error| format!("workflow {instance_id} failed: {error}"))?;

        let latency = started_at.elapsed();
        if let Err(error) = client.purge_workflow_state(&instance_id).await {
            eprintln!("Failed to purge workflow {instance_id}: {error}");
        }

        Ok(latency)
    });
}

fn print_summary(workflow_count: usize, latencies: &[Duration], elapsed: Duration) {
    let succeeded = latencies.len();
    let failed = workflow_count - succeeded;
    let throughput = succeeded as f64 / elapsed.as_secs_f64();

    let mut sorted_latencies = latencies.to_vec();
    sorted_latencies.sort_unstable();

    println!("Sustained workflow summary");
    println!("  total: {workflow_count}");
    println!("  succeeded: {succeeded}");
    println!("  failed: {failed}");
    println!("  total elapsed: {}", format_duration(elapsed));
    println!("  throughput: {throughput:.2} workflows/sec");

    if sorted_latencies.is_empty() {
        println!("  latency: no successful workflows");
        return;
    }

    println!("  latency avg: {}", format_duration(average(latencies)));
    println!("  latency min: {}", format_duration(sorted_latencies[0]));
    println!(
        "  latency p50: {}",
        format_duration(percentile(&sorted_latencies, 50.0))
    );
    println!(
        "  latency p95: {}",
        format_duration(percentile(&sorted_latencies, 95.0))
    );
    println!(
        "  latency p99: {}",
        format_duration(percentile(&sorted_latencies, 99.0))
    );
    println!(
        "  latency max: {}",
        format_duration(sorted_latencies[sorted_latencies.len() - 1])
    );
}

fn average(latencies: &[Duration]) -> Duration {
    Duration::from_secs_f64(
        latencies.iter().map(Duration::as_secs_f64).sum::<f64>() / latencies.len() as f64,
    )
}

fn percentile(sorted_latencies: &[Duration], percentile: f64) -> Duration {
    let index = ((percentile / 100.0) * (sorted_latencies.len() - 1) as f64).round() as usize;
    sorted_latencies[index]
}

fn format_duration(duration: Duration) -> String {
    format!("{:.2}ms", duration.as_secs_f64() * 1_000.0)
}
