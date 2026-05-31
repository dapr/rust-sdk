//! Dapr Workflows using the external crate `dapr-durabletask`.
//!
//! The workflow API follows this logic: register orchestrators and activities,
//! start a worker, schedule a workflow instance, wait for it to complete, then
//! purge its persisted history when it is no longer needed.
//! `WorkflowClient::new` connects to the Dapr sidecar using
//! `DAPR_GRPC_ENDPOINT`, `DAPR_GRPC_PORT`, or `http://127.0.0.1:50001`.
//!
//! # Quickstart
//!
//! Child workflows can receive caller history by using `SubWorkflowOptions` with
//! `HistoryPropagationScope`; both are re-exported from this module alongside
//! `PropagatedHistory` for consumers that inspect propagated context.
//!
//! ```rust,no_run
//! use dapr::workflow::{
//!     ActivityContext, ActivityContextExt, FetchOptions, HistoryPropagationScope, RegistryExt,
//!     ScheduleOptions, SubWorkflowOptions, WorkflowClient, WorkflowContext, WorkflowContextExt,
//! };
//!
//! async fn hello_workflow(ctx: WorkflowContext) -> dapr::workflow::Result<Option<String>> {
//!     let name: String = ctx.get_input_typed()?;
//!     let greeting: String = ctx.call_activity_typed("say_hello", name).await?;
//!     let _child_options =
//!         SubWorkflowOptions::new().with_history_propagation(HistoryPropagationScope::Lineage);
//!     Ok(Some(serde_json::to_string(&greeting)?))
//! }
//!
//! async fn say_hello(
//!     ctx: ActivityContext,
//!     input: Option<String>,
//! ) -> dapr::workflow::Result<Option<String>> {
//!     let name: String = ctx.get_input(input.as_deref())?;
//!     Ok(Some(serde_json::to_string(&format!("Hello, {name}!"))?))
//! }
//!
//! # async fn run() -> dapr::workflow::Result<()> {
//! let mut client = WorkflowClient::new().await?;
//!
//! client.registry_mut().add_workflow("hello", hello_workflow);
//! client.registry_mut().add_activity("say_hello", say_hello);
//!
//! let worker = client.start_worker().await?;
//! let id = client
//!     .schedule_workflow("hello", ScheduleOptions::new().with_input("Dapr"))
//!     .await?;
//! let metadata = client.wait_for_workflow_completion(&id).await?;
//! println!(
//!     "workflow {} finished as {}",
//!     metadata.instance_id, metadata.runtime_status
//! );
//! client
//!     .fetch_workflow_metadata(&id, FetchOptions::new().with_fetch_payloads(true))
//!     .await?;
//! client.purge_workflow_state(&id).await?;
//! worker.shutdown().await?;
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod context;
pub mod options;

pub use client::{WorkerHandle, WorkflowClient, WorkflowSchedulingClient};
pub use context::{
    ActivityContext, ActivityContextExt, RegistryExt, WorkflowContext, WorkflowContextExt,
};
pub use options::{
    ActivityOptions, EventOptions, FetchOptions, ScheduleOptions, SubWorkflowOptions,
};

pub use dapr_durabletask::api::{
    DurableTaskError as WorkflowError, FailureDetails, HistoryPropagationScope,
    OrchestrationState as WorkflowMetadata, OrchestrationStatus as RuntimeStatus,
    PropagatedHistory, PurgeInstanceFilter, Result, RetryPolicy,
};
pub use dapr_durabletask::task::{CompletableTask, OrchestrationContext, TaskResult, when_all, when_any};
pub use dapr_durabletask::worker::Registry;
