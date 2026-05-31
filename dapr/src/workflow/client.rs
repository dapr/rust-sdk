use std::time::Duration;

use dapr_durabletask::api::{DurableTaskError, OrchestrationState, PurgeInstanceFilter, Result};
use dapr_durabletask::client::TaskHubGrpcClient;
use dapr_durabletask::worker::{Registry, TaskHubGrpcWorker};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tonic::transport::Channel;

use super::options::{EventOptions, FetchOptions, ScheduleOptions};

/// Client for scheduling and managing Dapr workflow instances.
pub struct WorkflowClient {
    inner: TaskHubGrpcClient,
    channel: Channel,
    worker: Option<TaskHubGrpcWorker>,
}

impl WorkflowClient {
    /// Connect to the Dapr sidecar using `DAPR_GRPC_ENDPOINT`, `DAPR_GRPC_PORT`, or `127.0.0.1:50001`.
    pub async fn new() -> Result<Self> {
        Self::new_with_address(default_sidecar_address()).await
    }

    /// Connect to an explicit sidecar gRPC endpoint, for example `http://127.0.0.1:50001`.
    ///
    /// If `address` lacks a URL scheme, `http://` is prepended so that the
    /// underlying gRPC channel accepts it.
    ///
    /// # Arguments
    ///
    /// * `address` - Address of the Dapr sidecar gRPC endpoint to connect to.
    pub async fn new_with_address(address: impl Into<String>) -> Result<Self> {
        let address = ensure_url_scheme(address.into());
        let channel = Channel::from_shared(address.clone())
            .map_err(|e| DurableTaskError::InvalidAddress(e.to_string()))?
            .connect()
            .await
            .map_err(|e| DurableTaskError::ConnectionFailed(e.to_string()))?;
        let inner = TaskHubGrpcClient::from_channel(channel.clone());
        let worker = Some(TaskHubGrpcWorker::new(&address));
        Ok(Self {
            inner,
            channel,
            worker,
        })
    }

    /// Get the worker registry for registering workflows and activities before starting the worker.
    pub fn registry_mut(&mut self) -> &mut Registry {
        self.worker
            .as_mut()
            .expect("workflow worker registry is unavailable after start_worker has been called")
            .registry_mut()
    }

    /// Start the workflow worker on a Tokio task.
    pub async fn start_worker(&mut self) -> Result<WorkerHandle> {
        let worker = self.worker.take().ok_or_else(|| {
            DurableTaskError::Other("workflow worker has already been started".to_string())
        })?;
        let token = CancellationToken::new();
        let worker_token = token.clone();
        let join = tokio::spawn(async move { worker.start(worker_token).await });
        Ok(WorkerHandle {
            token,
            join: Some(join),
        })
    }

    /// Create a lightweight scheduling client that shares the underlying gRPC
    /// connection. Useful for high-throughput scenarios where many tasks need
    /// to schedule and manage workflows concurrently without opening new
    /// connections.
    ///
    /// The returned client is [`Clone`] — each clone reuses the same HTTP/2
    /// connection pool, so creating many clones is cheap.
    pub fn scheduling_client(&self) -> WorkflowSchedulingClient {
        WorkflowSchedulingClient {
            inner: TaskHubGrpcClient::from_channel(self.channel.clone()),
            channel: self.channel.clone(),
        }
    }

    /// Schedule a new workflow instance and return its instance ID.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the registered workflow to schedule.
    /// * `options` - Schedule options including input, instance ID, and start time.
    pub async fn schedule_workflow(
        &mut self,
        name: &str,
        options: ScheduleOptions,
    ) -> Result<String> {
        schedule_workflow_impl(&mut self.inner, name, options).await
    }

    /// Suspend a running workflow instance.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to suspend.
    /// * `reason` - Human-readable reason for suspending the workflow.
    pub async fn suspend_workflow(
        &mut self,
        instance_id: &str,
        reason: impl Into<String>,
    ) -> Result<()> {
        suspend_workflow_impl(&mut self.inner, instance_id, reason).await
    }

    /// Resume a suspended workflow instance.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to resume.
    /// * `reason` - Human-readable reason for resuming the workflow.
    pub async fn resume_workflow(
        &mut self,
        instance_id: &str,
        reason: impl Into<String>,
    ) -> Result<()> {
        resume_workflow_impl(&mut self.inner, instance_id, reason).await
    }

    /// Raise an event to a workflow instance.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to raise the event on.
    /// * `event_name` - Name of the event to raise.
    /// * `options` - Event options including the optional payload.
    pub async fn raise_event(
        &mut self,
        instance_id: &str,
        event_name: &str,
        options: EventOptions,
    ) -> Result<()> {
        raise_event_impl(&mut self.inner, instance_id, event_name, options).await
    }

    /// Fetch workflow metadata, optionally including inputs and outputs.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to fetch metadata for.
    /// * `options` - Fetch options controlling whether payloads are included.
    pub async fn fetch_workflow_metadata(
        &mut self,
        instance_id: &str,
        options: FetchOptions,
    ) -> Result<OrchestrationState> {
        fetch_workflow_metadata_impl(&mut self.inner, instance_id, options).await
    }

    /// Wait for a workflow to start.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to wait for.
    pub async fn wait_for_workflow_start(
        &mut self,
        instance_id: &str,
    ) -> Result<OrchestrationState> {
        wait_for_workflow_start_with_options_impl(&mut self.inner, instance_id, FetchOptions::new(), None)
            .await
    }

    /// Wait for a workflow to start with fetch and timeout options.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to wait for.
    /// * `options` - Fetch options controlling whether payloads are included.
    /// * `timeout` - Optional maximum time to wait before returning an error.
    pub async fn wait_for_workflow_start_with_options(
        &mut self,
        instance_id: &str,
        options: FetchOptions,
        timeout: Option<Duration>,
    ) -> Result<OrchestrationState> {
        wait_for_workflow_start_with_options_impl(&mut self.inner, instance_id, options, timeout)
            .await
    }

    /// Wait for a workflow to complete.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to wait for.
    pub async fn wait_for_workflow_completion(
        &mut self,
        instance_id: &str,
    ) -> Result<OrchestrationState> {
        wait_for_workflow_completion_with_options_impl(
            &mut self.inner,
            instance_id,
            FetchOptions::new(),
            None,
        )
        .await
    }

    /// Wait for a workflow to complete with fetch and timeout options.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to wait for.
    /// * `options` - Fetch options controlling whether payloads are included.
    /// * `timeout` - Optional maximum time to wait before returning an error.
    pub async fn wait_for_workflow_completion_with_options(
        &mut self,
        instance_id: &str,
        options: FetchOptions,
        timeout: Option<Duration>,
    ) -> Result<OrchestrationState> {
        wait_for_workflow_completion_with_options_impl(
            &mut self.inner,
            instance_id,
            options,
            timeout,
        )
        .await
    }

    /// Purge workflow state and history for an instance.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to purge.
    pub async fn purge_workflow_state(&mut self, instance_id: &str) -> Result<()> {
        purge_workflow_state_recursive_impl(&mut self.inner, instance_id, false).await
    }

    /// Purge workflow state and history for an instance and optionally child workflows.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to purge.
    /// * `recursive` - When `true`, also purge child workflow instances.
    pub async fn purge_workflow_state_recursive(
        &mut self,
        instance_id: &str,
        recursive: bool,
    ) -> Result<()> {
        purge_workflow_state_recursive_impl(&mut self.inner, instance_id, recursive).await
    }

    /// Purge workflow instances matching the given filter criteria.
    ///
    /// Returns the number of deleted instances.
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter criteria for selecting instances to purge.
    /// * `recursive` - When `true`, also purge child workflow instances.
    pub async fn purge_workflow_state_by_filter(
        &mut self,
        filter: PurgeInstanceFilter,
        recursive: bool,
    ) -> Result<i32> {
        purge_workflow_state_by_filter_impl(&mut self.inner, filter, recursive).await
    }

    /// Terminate a workflow instance.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to terminate.
    pub async fn terminate_workflow(&mut self, instance_id: &str) -> Result<()> {
        terminate_workflow_recursive_impl(&mut self.inner, instance_id, false).await
    }

    /// Terminate a workflow instance and optionally child workflows.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to terminate.
    /// * `recursive` - When `true`, also terminate child workflow instances.
    pub async fn terminate_workflow_recursive(
        &mut self,
        instance_id: &str,
        recursive: bool,
    ) -> Result<()> {
        terminate_workflow_recursive_impl(&mut self.inner, instance_id, recursive).await
    }
}

/// Lightweight, cloneable client for scheduling and managing workflow instances.
///
/// Created via [`WorkflowClient::scheduling_client`]. Every clone shares the
/// same underlying HTTP/2 connection pool, so creating many clones is cheap.
/// Use this when you need to schedule or wait on workflows from multiple
/// concurrent tasks without opening a new gRPC connection each time.
pub struct WorkflowSchedulingClient {
    inner: TaskHubGrpcClient,
    channel: Channel,
}

impl Clone for WorkflowSchedulingClient {
    fn clone(&self) -> Self {
        Self {
            inner: TaskHubGrpcClient::from_channel(self.channel.clone()),
            channel: self.channel.clone(),
        }
    }
}

impl WorkflowSchedulingClient {
    /// Schedule a new workflow instance and return its instance ID.
    pub async fn schedule_workflow(
        &mut self,
        name: &str,
        options: ScheduleOptions,
    ) -> Result<String> {
        schedule_workflow_impl(&mut self.inner, name, options).await
    }

    /// Suspend a running workflow instance.
    pub async fn suspend_workflow(
        &mut self,
        instance_id: &str,
        reason: impl Into<String>,
    ) -> Result<()> {
        suspend_workflow_impl(&mut self.inner, instance_id, reason).await
    }

    /// Resume a suspended workflow instance.
    pub async fn resume_workflow(
        &mut self,
        instance_id: &str,
        reason: impl Into<String>,
    ) -> Result<()> {
        resume_workflow_impl(&mut self.inner, instance_id, reason).await
    }

    /// Raise an event to a workflow instance.
    pub async fn raise_event(
        &mut self,
        instance_id: &str,
        event_name: &str,
        options: EventOptions,
    ) -> Result<()> {
        raise_event_impl(&mut self.inner, instance_id, event_name, options).await
    }

    /// Fetch workflow metadata, optionally including inputs and outputs.
    pub async fn fetch_workflow_metadata(
        &mut self,
        instance_id: &str,
        options: FetchOptions,
    ) -> Result<OrchestrationState> {
        fetch_workflow_metadata_impl(&mut self.inner, instance_id, options).await
    }

    /// Wait for a workflow to start.
    pub async fn wait_for_workflow_start(
        &mut self,
        instance_id: &str,
    ) -> Result<OrchestrationState> {
        wait_for_workflow_start_with_options_impl(
            &mut self.inner,
            instance_id,
            FetchOptions::new(),
            None,
        )
        .await
    }

    /// Wait for a workflow to start with fetch and timeout options.
    pub async fn wait_for_workflow_start_with_options(
        &mut self,
        instance_id: &str,
        options: FetchOptions,
        timeout: Option<Duration>,
    ) -> Result<OrchestrationState> {
        wait_for_workflow_start_with_options_impl(&mut self.inner, instance_id, options, timeout)
            .await
    }

    /// Wait for a workflow to complete.
    pub async fn wait_for_workflow_completion(
        &mut self,
        instance_id: &str,
    ) -> Result<OrchestrationState> {
        wait_for_workflow_completion_with_options_impl(
            &mut self.inner,
            instance_id,
            FetchOptions::new(),
            None,
        )
        .await
    }

    /// Wait for a workflow to complete with fetch and timeout options.
    pub async fn wait_for_workflow_completion_with_options(
        &mut self,
        instance_id: &str,
        options: FetchOptions,
        timeout: Option<Duration>,
    ) -> Result<OrchestrationState> {
        wait_for_workflow_completion_with_options_impl(
            &mut self.inner,
            instance_id,
            options,
            timeout,
        )
        .await
    }

    /// Purge workflow state and history for an instance.
    pub async fn purge_workflow_state(&mut self, instance_id: &str) -> Result<()> {
        purge_workflow_state_recursive_impl(&mut self.inner, instance_id, false).await
    }

    /// Purge workflow state and history for an instance and optionally child workflows.
    pub async fn purge_workflow_state_recursive(
        &mut self,
        instance_id: &str,
        recursive: bool,
    ) -> Result<()> {
        purge_workflow_state_recursive_impl(&mut self.inner, instance_id, recursive).await
    }

    /// Purge workflow instances matching the given filter criteria.
    ///
    /// Returns the number of deleted instances.
    pub async fn purge_workflow_state_by_filter(
        &mut self,
        filter: PurgeInstanceFilter,
        recursive: bool,
    ) -> Result<i32> {
        purge_workflow_state_by_filter_impl(&mut self.inner, filter, recursive).await
    }

    /// Terminate a workflow instance.
    pub async fn terminate_workflow(&mut self, instance_id: &str) -> Result<()> {
        terminate_workflow_recursive_impl(&mut self.inner, instance_id, false).await
    }

    /// Terminate a workflow instance and optionally child workflows.
    pub async fn terminate_workflow_recursive(
        &mut self,
        instance_id: &str,
        recursive: bool,
    ) -> Result<()> {
        terminate_workflow_recursive_impl(&mut self.inner, instance_id, recursive).await
    }
}

/// Handle for a running workflow worker.
///
/// Dropping the handle without calling [`WorkerHandle::shutdown`] requests
/// cancellation but does not await the worker task; the worker may continue to
/// drain in-flight work briefly before exiting. Prefer an explicit
/// `shutdown().await` for deterministic teardown.
pub struct WorkerHandle {
    token: CancellationToken,
    join: Option<JoinHandle<dapr_durabletask::api::Result<()>>>,
}

impl WorkerHandle {
    /// Request worker shutdown and wait for the worker task to finish.
    pub async fn shutdown(mut self) -> Result<()> {
        self.token.cancel();
        let join = self.join.take().ok_or_else(|| {
            DurableTaskError::Other("workflow worker task was already joined".to_string())
        })?;
        join.await
            .map_err(|e| DurableTaskError::Other(format!("workflow worker task failed: {e}")))?
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        self.token.cancel();
    }
}

fn default_sidecar_address() -> String {
    // Route through the shared client::config helper so behaviour stays in
    // sync with the main gRPC client.
    ensure_url_scheme(crate::client::config::default_sidecar_address())
}

/// Prepend `http://` to `address` when no URL scheme is present so that the
/// underlying tonic channel can parse it.
fn ensure_url_scheme(address: String) -> String {
    if address.contains("://") {
        address
    } else {
        format!("http://{address}")
    }
}

// ─── Shared scheduling / management helpers ─────────────────────────────────
// These free functions implement the actual logic, called by both
// `WorkflowClient` and `WorkflowSchedulingClient`.

async fn schedule_workflow_impl(
    client: &mut TaskHubGrpcClient,
    name: &str,
    options: ScheduleOptions,
) -> Result<String> {
    let instance_id = options.instance_id.clone();
    let start_time = options.start_time_utc();
    let input = options.input_json()?;
    client
        .schedule_new_orchestration(name, input, instance_id, start_time)
        .await
}

async fn suspend_workflow_impl(
    client: &mut TaskHubGrpcClient,
    instance_id: &str,
    reason: impl Into<String>,
) -> Result<()> {
    let reason = reason.into();
    client
        .suspend_orchestration(instance_id, Some(reason))
        .await
}

async fn resume_workflow_impl(
    client: &mut TaskHubGrpcClient,
    instance_id: &str,
    reason: impl Into<String>,
) -> Result<()> {
    let reason = reason.into();
    client
        .resume_orchestration(instance_id, Some(reason))
        .await
}

async fn raise_event_impl(
    client: &mut TaskHubGrpcClient,
    instance_id: &str,
    event_name: &str,
    options: EventOptions,
) -> Result<()> {
    client
        .raise_orchestration_event(instance_id, event_name, options.payload_json()?)
        .await
}

async fn fetch_workflow_metadata_impl(
    client: &mut TaskHubGrpcClient,
    instance_id: &str,
    options: FetchOptions,
) -> Result<OrchestrationState> {
    client
        .get_orchestration_state(instance_id, options.fetch_payloads)
        .await?
        .ok_or_else(|| DurableTaskError::InstanceNotFound {
            instance_id: instance_id.to_string(),
        })
}

async fn wait_for_workflow_start_with_options_impl(
    client: &mut TaskHubGrpcClient,
    instance_id: &str,
    options: FetchOptions,
    timeout: Option<Duration>,
) -> Result<OrchestrationState> {
    client
        .wait_for_orchestration_start(instance_id, options.fetch_payloads, timeout)
        .await?
        .ok_or_else(|| DurableTaskError::InstanceNotFound {
            instance_id: instance_id.to_string(),
        })
}

async fn wait_for_workflow_completion_with_options_impl(
    client: &mut TaskHubGrpcClient,
    instance_id: &str,
    options: FetchOptions,
    timeout: Option<Duration>,
) -> Result<OrchestrationState> {
    client
        .wait_for_orchestration_completion(instance_id, options.fetch_payloads, timeout)
        .await?
        .ok_or_else(|| DurableTaskError::InstanceNotFound {
            instance_id: instance_id.to_string(),
        })
}

async fn purge_workflow_state_recursive_impl(
    client: &mut TaskHubGrpcClient,
    instance_id: &str,
    recursive: bool,
) -> Result<()> {
    client
        .purge_orchestration(instance_id, recursive)
        .await?;
    Ok(())
}

async fn purge_workflow_state_by_filter_impl(
    client: &mut TaskHubGrpcClient,
    filter: PurgeInstanceFilter,
    recursive: bool,
) -> Result<i32> {
    client
        .purge_orchestrations_by_filter(filter, recursive)
        .await
}

async fn terminate_workflow_recursive_impl(
    client: &mut TaskHubGrpcClient,
    instance_id: &str,
    recursive: bool,
) -> Result<()> {
    client
        .terminate_orchestration(instance_id, None, recursive)
        .await
}

#[cfg(test)]
mod tests {
    use super::ensure_url_scheme;

    #[test]
    fn ensure_url_scheme_preserves_http() {
        assert_eq!(
            ensure_url_scheme("http://127.0.0.1:50001".to_string()),
            "http://127.0.0.1:50001"
        );
    }

    #[test]
    fn ensure_url_scheme_preserves_https() {
        assert_eq!(
            ensure_url_scheme("https://sidecar:443".to_string()),
            "https://sidecar:443"
        );
    }

    #[test]
    fn ensure_url_scheme_prepends_when_missing() {
        assert_eq!(
            ensure_url_scheme("sidecar:50001".to_string()),
            "http://sidecar:50001"
        );
    }

    #[test]
    fn ensure_url_scheme_prepends_for_bare_host() {
        assert_eq!(
            ensure_url_scheme("127.0.0.1".to_string()),
            "http://127.0.0.1"
        );
    }
}
