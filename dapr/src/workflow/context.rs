use std::future::Future;
use std::time::Duration;

use dapr_durabletask::worker::{ActivityResult, OrchestratorResult, Registry};
use futures::future::Either;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::options::{ActivityOptions, SubWorkflowOptions};

pub type WorkflowContext = dapr_durabletask::task::OrchestrationContext;
pub type ActivityContext = dapr_durabletask::task::ActivityContext;

/// Typed convenience helpers for workflow orchestrators.
pub trait WorkflowContextExt {
    /// Deserialize the workflow input into a typed value.
    fn get_input_typed<T: DeserializeOwned>(&self) -> dapr_durabletask::api::Result<T>;

    /// Call an activity by name with a JSON-serializable input and deserialize the result.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the registered activity to invoke.
    /// * `input` - Value serialized to JSON and delivered as the activity input.
    fn call_activity_typed<T, I>(
        &self,
        name: &str,
        input: I,
    ) -> impl Future<Output = dapr_durabletask::api::Result<T>> + Send + 'static
    where
        T: DeserializeOwned + Send + 'static,
        I: Serialize + Send + 'static;

    /// Call an activity by name with explicit options and deserialize the result.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the registered activity to invoke.
    /// * `options` - Activity options including input, retry policy, and history propagation.
    fn call_activity_with_options_typed<T>(
        &self,
        name: &str,
        options: ActivityOptions,
    ) -> impl Future<Output = dapr_durabletask::api::Result<T>> + Send + 'static
    where
        T: DeserializeOwned + Send + 'static;

    /// Call a child workflow by name with a JSON-serializable input and deserialize the result.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the registered child workflow to invoke.
    /// * `input` - Value serialized to JSON and delivered as the child workflow input.
    fn call_sub_workflow_typed<T, I>(
        &self,
        name: &str,
        input: I,
    ) -> impl Future<Output = dapr_durabletask::api::Result<T>> + Send + 'static
    where
        T: DeserializeOwned + Send + 'static,
        I: Serialize + Send + 'static;

    /// Call a child workflow by name with explicit options and deserialize the result.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the registered child workflow to invoke.
    /// * `options` - Child workflow options including input, instance ID, app ID, retry policy, and history propagation.
    fn call_sub_workflow_with_options_typed<T>(
        &self,
        name: &str,
        options: SubWorkflowOptions,
    ) -> impl Future<Output = dapr_durabletask::api::Result<T>> + Send + 'static
    where
        T: DeserializeOwned + Send + 'static;

    /// Return the propagated history attached to the current workflow invocation, if any.
    fn propagated_history(
        &self,
    ) -> Option<std::sync::Arc<dapr_durabletask::api::PropagatedHistory>>;

    /// Wait for an external event with an optional timeout and deserialize the payload.
    ///
    /// When `timeout` is `Some`, a durable timer is scheduled alongside the
    /// external event wait. The underlying timer cannot be cancelled in
    /// `dapr-durabletask 0.0.1`, so the timer event is persisted into history
    /// even when the external event arrives first; this is the same behaviour
    /// as other Durable Task SDKs.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the external event to wait for.
    /// * `timeout` - Optional maximum time to wait before returning `DurableTaskError::Timeout`.
    fn wait_for_external_event_typed<T>(
        &self,
        name: &str,
        timeout: Option<Duration>,
    ) -> impl Future<Output = dapr_durabletask::api::Result<T>> + Send + 'static
    where
        T: DeserializeOwned + Send + 'static;
}

impl WorkflowContextExt for WorkflowContext {
    fn get_input_typed<T: DeserializeOwned>(&self) -> dapr_durabletask::api::Result<T> {
        self.input()
    }

    fn call_activity_typed<T, I>(
        &self,
        name: &str,
        input: I,
    ) -> impl Future<Output = dapr_durabletask::api::Result<T>> + Send + 'static
    where
        T: DeserializeOwned + Send + 'static,
        I: Serialize + Send + 'static,
    {
        let task = self.call_activity(name, input);
        async move { deserialize_task_output(task.await?) }
    }

    fn call_activity_with_options_typed<T>(
        &self,
        name: &str,
        options: ActivityOptions,
    ) -> impl Future<Output = dapr_durabletask::api::Result<T>> + Send + 'static
    where
        T: DeserializeOwned + Send + 'static,
    {
        let (input, task_options) = options.into_parts();
        let task = self.call_activity_with_options(name, input, task_options);
        async move { deserialize_task_output(task.await?) }
    }

    fn call_sub_workflow_typed<T, I>(
        &self,
        name: &str,
        input: I,
    ) -> impl Future<Output = dapr_durabletask::api::Result<T>> + Send + 'static
    where
        T: DeserializeOwned + Send + 'static,
        I: Serialize + Send + 'static,
    {
        let task = self.call_sub_orchestrator_with_options(
            name,
            input,
            dapr_durabletask::task::SubOrchestratorOptions::new(),
        );
        async move { deserialize_task_output(task.await?) }
    }

    fn call_sub_workflow_with_options_typed<T>(
        &self,
        name: &str,
        options: SubWorkflowOptions,
    ) -> impl Future<Output = dapr_durabletask::api::Result<T>> + Send + 'static
    where
        T: DeserializeOwned + Send + 'static,
    {
        let (input, task_options) = options.into_parts();
        let task = self.call_sub_orchestrator_with_options(name, input, task_options);
        async move { deserialize_task_output(task.await?) }
    }

    fn propagated_history(
        &self,
    ) -> Option<std::sync::Arc<dapr_durabletask::api::PropagatedHistory>> {
        dapr_durabletask::task::OrchestrationContext::propagated_history(self)
    }

    fn wait_for_external_event_typed<T>(
        &self,
        name: &str,
        timeout: Option<Duration>,
    ) -> impl Future<Output = dapr_durabletask::api::Result<T>> + Send + 'static
    where
        T: DeserializeOwned + Send + 'static,
    {
        let event = self.wait_for_external_event(name);
        let timer = timeout.map(|duration| self.create_timer(duration));
        async move {
            let output = match timer {
                Some(timer) => {
                    futures::pin_mut!(event);
                    futures::pin_mut!(timer);
                    match futures::future::select(event, timer).await {
                        Either::Left((event_result, _)) => event_result?,
                        Either::Right(_) => {
                            return Err(dapr_durabletask::api::DurableTaskError::Timeout);
                        }
                    }
                }
                None => event.await?,
            };
            deserialize_task_output(output)
        }
    }
}

/// Typed convenience helpers for activity functions.
pub trait ActivityContextExt {
    /// Deserialize the activity input string into a typed value.
    ///
    /// # Arguments
    ///
    /// * `input` - Raw JSON input string passed to the activity, or `None` for a null input.
    fn get_input<T: DeserializeOwned>(
        &self,
        input: Option<&str>,
    ) -> dapr_durabletask::api::Result<T>;

    /// Return the propagated history attached to the current activity invocation, if any.
    fn propagated_history(&self) -> Option<&dapr_durabletask::api::PropagatedHistory>;
}

impl ActivityContextExt for ActivityContext {
    fn get_input<T: DeserializeOwned>(
        &self,
        input: Option<&str>,
    ) -> dapr_durabletask::api::Result<T> {
        deserialize_task_output(input.map(ToOwned::to_owned))
    }

    fn propagated_history(&self) -> Option<&dapr_durabletask::api::PropagatedHistory> {
        dapr_durabletask::task::ActivityContext::propagated_history(self)
    }
}

/// Go-style aliases for registering workflows and activities on a durable task registry.
pub trait RegistryExt {
    /// Register a workflow orchestrator function under the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - Name used to schedule and invoke the workflow.
    /// * `f` - Orchestrator function invoked with a [`WorkflowContext`].
    fn add_workflow<F, Fut>(&mut self, name: &str, f: F)
    where
        F: Fn(WorkflowContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = OrchestratorResult> + Send + 'static;

    /// Register an activity function under the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - Name used to invoke the activity from a workflow.
    /// * `f` - Activity function invoked with an [`ActivityContext`] and optional raw JSON input.
    fn add_activity<F, Fut>(&mut self, name: &str, f: F)
    where
        F: Fn(ActivityContext, Option<String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ActivityResult> + Send + 'static;
}

impl RegistryExt for Registry {
    fn add_workflow<F, Fut>(&mut self, name: &str, f: F)
    where
        F: Fn(WorkflowContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = OrchestratorResult> + Send + 'static,
    {
        self.add_named_orchestrator(name, f);
    }

    fn add_activity<F, Fut>(&mut self, name: &str, f: F)
    where
        F: Fn(ActivityContext, Option<String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ActivityResult> + Send + 'static,
    {
        self.add_named_activity(name, f);
    }
}

fn deserialize_task_output<T: DeserializeOwned>(
    output: Option<String>,
) -> dapr_durabletask::api::Result<T> {
    match output {
        Some(value) => serde_json::from_str(&value).map_err(Into::into),
        None => serde_json::from_value(serde_json::Value::Null).map_err(Into::into),
    }
}
