use std::env;
use std::time::Duration;

use dapr_durabletask::api::{DurableTaskError, OrchestrationState, Result};
use dapr_durabletask::client::TaskHubGrpcClient;
use dapr_durabletask::worker::{Registry, TaskHubGrpcWorker};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::options::{EventOptions, FetchOptions, ScheduleOptions};

/// Client for scheduling and managing Dapr workflow instances.
pub struct WorkflowClient {
    inner: TaskHubGrpcClient,
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
        let inner = TaskHubGrpcClient::new(&address).await?;
        let worker = Some(TaskHubGrpcWorker::new(&address));
        Ok(Self { inner, worker })
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
        let instance_id = options.instance_id.clone();
        let start_time = options.start_time_utc();
        let input = options.input_json()?;
        self.inner
            .schedule_new_orchestration(name, input, instance_id, start_time)
            .await
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
        let reason = reason.into();
        self.inner
            .suspend_orchestration(instance_id, Some(reason))
            .await
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
        let reason = reason.into();
        self.inner
            .resume_orchestration(instance_id, Some(reason))
            .await
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
        self.inner
            .raise_orchestration_event(instance_id, event_name, options.payload_json()?)
            .await
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
        self.inner
            .get_orchestration_state(instance_id, options.fetch_payloads)
            .await?
            .ok_or_else(|| DurableTaskError::InstanceNotFound {
                instance_id: instance_id.to_string(),
            })
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
        self.wait_for_workflow_start_with_options(instance_id, FetchOptions::new(), None)
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
        self.inner
            .wait_for_orchestration_start(instance_id, options.fetch_payloads, timeout)
            .await?
            .ok_or_else(|| DurableTaskError::InstanceNotFound {
                instance_id: instance_id.to_string(),
            })
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
        self.wait_for_workflow_completion_with_options(instance_id, FetchOptions::new(), None)
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
        self.inner
            .wait_for_orchestration_completion(instance_id, options.fetch_payloads, timeout)
            .await?
            .ok_or_else(|| DurableTaskError::InstanceNotFound {
                instance_id: instance_id.to_string(),
            })
    }

    /// Purge workflow state and history for an instance.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to purge.
    pub async fn purge_workflow_state(&mut self, instance_id: &str) -> Result<()> {
        self.purge_workflow_state_recursive(instance_id, false)
            .await
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
        self.inner
            .purge_orchestration(instance_id, recursive)
            .await?;
        Ok(())
    }

    /// Terminate a workflow instance.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - ID of the workflow instance to terminate.
    pub async fn terminate_workflow(&mut self, instance_id: &str) -> Result<()> {
        self.terminate_workflow_recursive(instance_id, false).await
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
        self.inner
            .terminate_orchestration(instance_id, None, recursive)
            .await
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
    if let Ok(endpoint) = env::var("DAPR_GRPC_ENDPOINT") {
        return ensure_url_scheme(endpoint);
    }

    match env::var("DAPR_GRPC_PORT") {
        Ok(port) => format!("http://127.0.0.1:{port}"),
        Err(_) => "http://127.0.0.1:50001".to_string(),
    }
}

/// Prepend `http://` to `address` when no URL scheme is present so that the
/// underlying tonic channel can parse it.
///
/// `dapr-durabletask 0.0.1` exposes no token interceptor or per-request metadata
/// hook, so `DAPR_API_TOKEN` cannot be forwarded from this layer yet. Once the
/// upstream client gains an interceptor surface, wire token injection here.
fn ensure_url_scheme(address: String) -> String {
    if address.contains("://") {
        address
    } else {
        format!("http://{address}")
    }
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
