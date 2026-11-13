use std::time::SystemTime;

use dapr_durabletask::api::{HistoryPropagationScope, RetryPolicy};
use serde::Serialize;

/// Options for scheduling a new workflow instance.
#[derive(Debug, Clone, Default)]
pub struct ScheduleOptions {
    /// Optional caller-supplied instance ID. When `None`, the sidecar generates one.
    pub instance_id: Option<String>,
    /// Optional JSON-serialized input passed to the workflow.
    pub input: Option<serde_json::Value>,
    /// Optional start time. When `None`, the workflow starts immediately.
    pub start_time: Option<SystemTime>,
}

impl ScheduleOptions {
    /// Create an empty set of schedule options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set an explicit instance ID for the new workflow.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - Caller-supplied identifier for the workflow instance.
    pub fn with_instance_id(mut self, instance_id: impl Into<String>) -> Self {
        self.instance_id = Some(instance_id.into());
        self
    }

    /// Set the JSON-serializable input passed to the workflow.
    ///
    /// # Panics
    ///
    /// Panics if `input` cannot be serialized to JSON. Prefer
    /// [`ScheduleOptions::try_with_input`] for fallible serialization.
    ///
    /// # Arguments
    ///
    /// * `input` - Value serialized to JSON and delivered as the workflow input.
    pub fn with_input<T: Serialize>(self, input: T) -> Self {
        self.try_with_input(input)
            .expect("workflow input must be JSON serializable")
    }

    /// Set the JSON-serializable input passed to the workflow, returning an
    /// error if serialization fails.
    ///
    /// # Arguments
    ///
    /// * `input` - Value serialized to JSON and delivered as the workflow input.
    pub fn try_with_input<T: Serialize>(mut self, input: T) -> Result<Self, serde_json::Error> {
        self.input = Some(serde_json::to_value(input)?);
        Ok(self)
    }

    /// Set the scheduled start time for the workflow.
    ///
    /// # Arguments
    ///
    /// * `start_time` - Earliest time at which the workflow should begin executing.
    pub fn with_start_time(mut self, start_time: SystemTime) -> Self {
        self.start_time = Some(start_time);
        self
    }

    pub(crate) fn input_json(self) -> dapr_durabletask::api::Result<Option<String>> {
        self.input
            .map(|value| serde_json::to_string(&value))
            .transpose()
            .map_err(Into::into)
    }

    pub(crate) fn start_time_utc(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.start_time.map(chrono::DateTime::<chrono::Utc>::from)
    }
}

/// Options for creating a workflow schedule.
///
/// # Examples
///
/// ```rust
/// use std::time::{Duration, SystemTime};
///
/// use dapr::workflow::CreateWorkflowScheduleOptions;
///
/// let opts = CreateWorkflowScheduleOptions::new()
///     .with_input(serde_json::json!({ "region": "eu" }))
///     .with_expiration_time(SystemTime::now() + Duration::from_secs(86_400))
///     .with_repeats(10)
///     .with_overwrite();
/// assert!(opts.overwrite);
/// ```
#[derive(Debug, Clone, Default)]
pub struct CreateWorkflowScheduleOptions {
    /// Optional JSON-serialized input given to every run.
    pub input: Option<serde_json::Value>,
    /// Optional time of the first run. When `None`, the first run is at the first time the
    /// schedule matches after it is created.
    pub due_time: Option<SystemTime>,
    /// Optional time after which no run starts. The schedule completes once its next run would
    /// be after it.
    pub expiration_time: Option<SystemTime>,
    /// Optional number of runs after which the schedule completes. Must be at least 1.
    pub repeats: Option<u32>,
    /// Optional app ID hosting the workflow each run starts. When `None`, the app that owns the
    /// schedule.
    pub target_app_id: Option<String>,
    /// Optional namespace of the target app ID. Requires `target_app_id`.
    pub target_app_namespace: Option<String>,
    /// When `true`, replace an existing schedule of the same name. Runs it already started are
    /// not affected. Otherwise creating a schedule whose name is taken fails with gRPC status
    /// `ALREADY_EXISTS`.
    pub overwrite: bool,
    /// When `true`, start one run for the times the schedule missed while the app was down, then
    /// carry on from the next time after now. When `false` (the default), the schedule catches
    /// up: every missed time starts its own run, one after another, as with Dapr jobs and actor
    /// reminders.
    pub skip_missed: bool,
}

impl CreateWorkflowScheduleOptions {
    /// Create an empty set of workflow schedule options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the JSON-serializable input given to every run.
    ///
    /// # Panics
    ///
    /// Panics if `input` cannot be serialized to JSON. Prefer
    /// [`CreateWorkflowScheduleOptions::try_with_input`] for fallible serialization.
    ///
    /// # Arguments
    ///
    /// * `input` - Value serialized to JSON and delivered as the input of every run.
    pub fn with_input<T: Serialize>(self, input: T) -> Self {
        self.try_with_input(input)
            .expect("workflow schedule input must be JSON serializable")
    }

    /// Set the JSON-serializable input given to every run, returning an error if serialization
    /// fails.
    ///
    /// # Arguments
    ///
    /// * `input` - Value serialized to JSON and delivered as the input of every run.
    pub fn try_with_input<T: Serialize>(mut self, input: T) -> Result<Self, serde_json::Error> {
        self.input = Some(serde_json::to_value(input)?);
        Ok(self)
    }

    /// Set the time of the first run.
    ///
    /// # Arguments
    ///
    /// * `due_time` - Time at which the first run starts.
    pub fn with_due_time(mut self, due_time: SystemTime) -> Self {
        self.due_time = Some(due_time);
        self
    }

    /// Set the time after which no run starts.
    ///
    /// # Arguments
    ///
    /// * `expiration_time` - Time after which the schedule starts no more runs.
    pub fn with_expiration_time(mut self, expiration_time: SystemTime) -> Self {
        self.expiration_time = Some(expiration_time);
        self
    }

    /// Set the number of runs after which the schedule completes.
    ///
    /// # Arguments
    ///
    /// * `repeats` - Number of runs, at least 1.
    pub fn with_repeats(mut self, repeats: u32) -> Self {
        self.repeats = Some(repeats);
        self
    }

    /// Set the app ID hosting the workflow each run starts.
    ///
    /// # Arguments
    ///
    /// * `app_id` - Dapr app ID that should execute each run.
    pub fn with_target_app_id(mut self, app_id: impl Into<String>) -> Self {
        self.target_app_id = Some(app_id.into());
        self
    }

    /// Set the namespace of the target app ID.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace of the target app ID.
    pub fn with_target_app_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.target_app_namespace = Some(namespace.into());
        self
    }

    /// Replace an existing schedule of the same name.
    pub fn with_overwrite(mut self) -> Self {
        self.overwrite = true;
        self
    }

    /// Start one run for the times the schedule missed while the app was down instead of one run
    /// for each of them. See [`CreateWorkflowScheduleOptions::skip_missed`].
    pub fn with_skip_missed(mut self) -> Self {
        self.skip_missed = true;
        self
    }

    pub(crate) fn into_durabletask(
        self,
    ) -> dapr_durabletask::api::Result<dapr_durabletask::client::CreateWorkflowScheduleOptions>
    {
        Ok(dapr_durabletask::client::CreateWorkflowScheduleOptions {
            input: self
                .input
                .map(|value| serde_json::to_string(&value))
                .transpose()?,
            due_time: self.due_time.map(chrono::DateTime::<chrono::Utc>::from),
            expiration_time: self
                .expiration_time
                .map(chrono::DateTime::<chrono::Utc>::from),
            repeats: self.repeats,
            target_app_id: self.target_app_id,
            target_app_namespace: self.target_app_namespace,
            overwrite: self.overwrite,
            skip_missed: self.skip_missed,
        })
    }
}

/// Options for calling an activity from a workflow.
#[derive(Debug, Clone, Default)]
pub struct ActivityOptions {
    /// Optional JSON-serialized input passed to the activity.
    pub input: Option<serde_json::Value>,
    /// Optional retry policy applied to the activity invocation.
    pub retry_policy: Option<RetryPolicy>,
    /// Optional scope controlling how caller history is propagated to the activity.
    pub history_propagation: Option<HistoryPropagationScope>,
}

impl ActivityOptions {
    /// Create an empty set of activity options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the JSON-serializable input passed to the activity.
    ///
    /// # Panics
    ///
    /// Panics if `input` cannot be serialized to JSON. Prefer
    /// [`ActivityOptions::try_with_input`] for fallible serialization.
    ///
    /// # Arguments
    ///
    /// * `input` - Value serialized to JSON and delivered as the activity input.
    pub fn with_input<T: Serialize>(self, input: T) -> Self {
        self.try_with_input(input)
            .expect("activity input must be JSON serializable")
    }

    /// Set the JSON-serializable input passed to the activity, returning an
    /// error if serialization fails.
    ///
    /// # Arguments
    ///
    /// * `input` - Value serialized to JSON and delivered as the activity input.
    pub fn try_with_input<T: Serialize>(mut self, input: T) -> Result<Self, serde_json::Error> {
        self.input = Some(serde_json::to_value(input)?);
        Ok(self)
    }

    /// Set the retry policy applied to the activity invocation.
    ///
    /// # Arguments
    ///
    /// * `retry_policy` - Policy describing retry attempts, intervals, and backoff.
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }

    /// Set the history propagation scope applied to the activity invocation.
    ///
    /// # Arguments
    ///
    /// * `scope` - Scope controlling how caller history is propagated to the activity.
    pub fn with_history_propagation(mut self, scope: HistoryPropagationScope) -> Self {
        self.history_propagation = Some(scope);
        self
    }

    pub(crate) fn into_parts(self) -> (serde_json::Value, dapr_durabletask::task::ActivityOptions) {
        let input = self.input.unwrap_or(serde_json::Value::Null);
        let mut options = dapr_durabletask::task::ActivityOptions::new();
        if let Some(policy) = self.retry_policy {
            options = options.with_retry_policy(policy);
        }
        if let Some(scope) = self.history_propagation {
            options = options.with_history_propagation(scope);
        }
        (input, options)
    }
}

/// Options for calling a child workflow from a workflow.
#[derive(Debug, Clone, Default)]
pub struct SubWorkflowOptions {
    /// Optional JSON-serialized input passed to the child workflow.
    pub input: Option<serde_json::Value>,
    /// Optional caller-supplied instance ID for the child workflow.
    pub instance_id: Option<String>,
    /// Optional target app ID hosting the child workflow.
    pub app_id: Option<String>,
    /// Optional retry policy applied to the child workflow invocation.
    pub retry_policy: Option<RetryPolicy>,
    /// Optional scope controlling how caller history is propagated to the child workflow.
    pub history_propagation: Option<HistoryPropagationScope>,
}

impl SubWorkflowOptions {
    /// Create an empty set of child workflow options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the JSON-serializable input passed to the child workflow.
    ///
    /// # Panics
    ///
    /// Panics if `input` cannot be serialized to JSON. Prefer
    /// [`SubWorkflowOptions::try_with_input`] for fallible serialization.
    ///
    /// # Arguments
    ///
    /// * `input` - Value serialized to JSON and delivered as the child workflow input.
    pub fn with_input<T: Serialize>(self, input: T) -> Self {
        self.try_with_input(input)
            .expect("child workflow input must be JSON serializable")
    }

    /// Set the JSON-serializable input passed to the child workflow, returning
    /// an error if serialization fails.
    ///
    /// # Arguments
    ///
    /// * `input` - Value serialized to JSON and delivered as the child workflow input.
    pub fn try_with_input<T: Serialize>(mut self, input: T) -> Result<Self, serde_json::Error> {
        self.input = Some(serde_json::to_value(input)?);
        Ok(self)
    }

    /// Set an explicit instance ID for the child workflow.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - Caller-supplied identifier for the child workflow instance.
    pub fn with_instance_id(mut self, instance_id: impl Into<String>) -> Self {
        self.instance_id = Some(instance_id.into());
        self
    }

    /// Set the target app ID hosting the child workflow.
    ///
    /// # Arguments
    ///
    /// * `app_id` - Dapr app ID that should execute the child workflow.
    pub fn with_app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    /// Set the retry policy applied to the child workflow invocation.
    ///
    /// # Arguments
    ///
    /// * `retry_policy` - Policy describing retry attempts, intervals, and backoff.
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }

    /// Set the history propagation scope applied to the child workflow invocation.
    ///
    /// # Arguments
    ///
    /// * `scope` - Scope controlling how caller history is propagated to the child workflow.
    pub fn with_history_propagation(mut self, scope: HistoryPropagationScope) -> Self {
        self.history_propagation = Some(scope);
        self
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        serde_json::Value,
        dapr_durabletask::task::SubOrchestratorOptions,
    ) {
        let input = self.input.unwrap_or(serde_json::Value::Null);
        let mut options = dapr_durabletask::task::SubOrchestratorOptions::new();
        if let Some(instance_id) = self.instance_id {
            options = options.with_instance_id(instance_id);
        }
        if let Some(app_id) = self.app_id {
            options = options.with_app_id(app_id);
        }
        if let Some(policy) = self.retry_policy {
            options = options.with_retry_policy(policy);
        }
        if let Some(scope) = self.history_propagation {
            options = options.with_history_propagation(scope);
        }
        (input, options)
    }
}

/// Options for raising an event to a workflow instance.
#[derive(Debug, Clone, Default)]
pub struct EventOptions {
    /// Optional JSON-serialized payload delivered with the event.
    pub payload: Option<serde_json::Value>,
}

impl EventOptions {
    /// Create an empty set of event options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the JSON-serializable payload delivered with the event.
    ///
    /// # Panics
    ///
    /// Panics if `payload` cannot be serialized to JSON. Prefer
    /// [`EventOptions::try_with_payload`] for fallible serialization.
    ///
    /// # Arguments
    ///
    /// * `payload` - Value serialized to JSON and delivered as the event payload.
    pub fn with_payload<T: Serialize>(self, payload: T) -> Self {
        self.try_with_payload(payload)
            .expect("event payload must be JSON serializable")
    }

    /// Set the JSON-serializable payload delivered with the event, returning
    /// an error if serialization fails.
    ///
    /// # Arguments
    ///
    /// * `payload` - Value serialized to JSON and delivered as the event payload.
    pub fn try_with_payload<T: Serialize>(mut self, payload: T) -> Result<Self, serde_json::Error> {
        self.payload = Some(serde_json::to_value(payload)?);
        Ok(self)
    }

    pub(crate) fn payload_json(self) -> dapr_durabletask::api::Result<Option<String>> {
        self.payload
            .map(|value| serde_json::to_string(&value))
            .transpose()
            .map_err(Into::into)
    }
}

/// Options for fetching workflow metadata.
#[derive(Debug, Clone, Copy, Default)]
pub struct FetchOptions {
    /// When `true`, include workflow inputs and outputs in the returned metadata.
    pub fetch_payloads: bool,
}

impl FetchOptions {
    /// Create an empty set of fetch options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether workflow inputs and outputs are included in the returned metadata.
    ///
    /// # Arguments
    ///
    /// * `fetch_payloads` - When `true`, include workflow inputs and outputs.
    pub fn with_fetch_payloads(mut self, fetch_payloads: bool) -> Self {
        self.fetch_payloads = fetch_payloads;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::ser::Error as _;
    use serde::{Deserialize, Serialize, Serializer};
    use std::time::{Duration, UNIX_EPOCH};

    #[derive(Serialize, Deserialize)]
    struct Payload {
        name: String,
        count: u32,
    }

    struct AlwaysFails;

    impl Serialize for AlwaysFails {
        fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(S::Error::custom("intentional serialization failure"))
        }
    }

    #[test]
    fn schedule_options_default_input_json_is_none() {
        let opts = ScheduleOptions::new();
        assert!(opts.input_json().unwrap().is_none());
    }

    #[test]
    fn schedule_options_input_json_round_trips_struct() {
        let opts = ScheduleOptions::new().with_input(Payload {
            name: "hello".into(),
            count: 3,
        });
        let json = opts.input_json().unwrap().expect("input present");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["name"], "hello");
        assert_eq!(v["count"], 3);
    }

    #[test]
    fn schedule_options_input_json_preserves_escapes() {
        let opts = ScheduleOptions::new().with_input("line1\nline2\t\"quoted\"");
        let json = opts.input_json().unwrap().expect("input present");
        let decoded: String = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, "line1\nline2\t\"quoted\"");
    }

    #[test]
    fn schedule_options_try_with_input_propagates_error() {
        let result = ScheduleOptions::new().try_with_input(AlwaysFails);
        assert!(result.is_err());
    }

    #[test]
    fn schedule_options_start_time_utc_handles_epoch() {
        let opts = ScheduleOptions::new().with_start_time(UNIX_EPOCH);
        let dt = opts.start_time_utc().expect("start time set");
        assert_eq!(dt.timestamp(), 0);
    }

    #[test]
    fn schedule_options_start_time_utc_none_by_default() {
        assert!(ScheduleOptions::new().start_time_utc().is_none());
    }

    #[test]
    fn event_options_payload_json_handles_none() {
        assert!(EventOptions::new().payload_json().unwrap().is_none());
    }

    #[test]
    fn event_options_payload_json_round_trips() {
        let opts = EventOptions::new().with_payload(vec![1, 2, 3]);
        let json = opts.payload_json().unwrap().expect("payload present");
        let v: Vec<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn event_options_try_with_payload_propagates_error() {
        let result = EventOptions::new().try_with_payload(AlwaysFails);
        assert!(result.is_err());
    }

    #[test]
    fn activity_options_into_parts_defaults_to_null_input() {
        let (input, _) = ActivityOptions::new().into_parts();
        assert!(input.is_null());
    }

    #[test]
    fn activity_options_into_parts_carries_input() {
        let (input, _) = ActivityOptions::new().with_input(42_i32).into_parts();
        assert_eq!(input, serde_json::json!(42));
    }

    #[test]
    fn activity_options_into_parts_with_retry_and_propagation_compiles() {
        // We can't introspect the lowered ActivityOptions, but we can verify
        // the call chain accepts all builder methods and consumes self once.
        let policy = RetryPolicy::new(2, Duration::from_millis(10));
        let (_input, _task_opts) = ActivityOptions::new()
            .with_input("payload")
            .with_retry_policy(policy)
            .with_history_propagation(HistoryPropagationScope::OwnHistory)
            .into_parts();
    }

    #[test]
    fn sub_workflow_options_into_parts_defaults_to_null_input() {
        let (input, _) = SubWorkflowOptions::new().into_parts();
        assert!(input.is_null());
    }

    #[test]
    fn sub_workflow_options_into_parts_carries_input() {
        let (input, _) = SubWorkflowOptions::new()
            .with_input(Payload {
                name: "child".into(),
                count: 7,
            })
            .into_parts();
        assert_eq!(input["name"], "child");
        assert_eq!(input["count"], 7);
    }

    #[test]
    fn sub_workflow_options_full_builder_compiles() {
        let policy = RetryPolicy::new(3, Duration::from_millis(50));
        let (_input, _task_opts) = SubWorkflowOptions::new()
            .with_input("payload")
            .with_instance_id("child-id")
            .with_app_id("worker-app")
            .with_retry_policy(policy)
            .with_history_propagation(HistoryPropagationScope::Lineage)
            .into_parts();
    }

    #[test]
    fn create_workflow_schedule_options_into_durabletask_carries_every_option() {
        let opts = CreateWorkflowScheduleOptions::new()
            .with_input(Payload {
                name: "nightly".into(),
                count: 2,
            })
            .with_due_time(UNIX_EPOCH + Duration::from_secs(1))
            .with_expiration_time(UNIX_EPOCH + Duration::from_secs(2))
            .with_repeats(3)
            .with_target_app_id("worker-app")
            .with_target_app_namespace("ns")
            .with_overwrite()
            .with_skip_missed()
            .into_durabletask()
            .unwrap();
        let v: serde_json::Value = serde_json::from_str(opts.input.as_deref().unwrap()).unwrap();
        assert_eq!(v["name"], "nightly");
        assert_eq!(v["count"], 2);
        assert_eq!(opts.due_time.unwrap().timestamp(), 1);
        assert_eq!(opts.expiration_time.unwrap().timestamp(), 2);
        assert_eq!(opts.repeats, Some(3));
        assert_eq!(opts.target_app_id.as_deref(), Some("worker-app"));
        assert_eq!(opts.target_app_namespace.as_deref(), Some("ns"));
        assert!(opts.overwrite);
        assert!(opts.skip_missed);
    }

    #[test]
    fn create_workflow_schedule_options_default_into_durabletask_is_empty() {
        let opts = CreateWorkflowScheduleOptions::new()
            .into_durabletask()
            .unwrap();
        assert!(opts.input.is_none() && opts.due_time.is_none() && opts.expiration_time.is_none());
        assert!(opts.repeats.is_none() && opts.target_app_id.is_none());
        assert!(opts.target_app_namespace.is_none() && !opts.overwrite && !opts.skip_missed);
    }

    #[test]
    fn create_workflow_schedule_options_try_with_input_propagates_error() {
        let result = CreateWorkflowScheduleOptions::new().try_with_input(AlwaysFails);
        assert!(result.is_err());
    }

    #[test]
    fn fetch_options_with_fetch_payloads_toggles() {
        let opts = FetchOptions::new();
        assert!(!opts.fetch_payloads);
        let opts = opts.with_fetch_payloads(true);
        assert!(opts.fetch_payloads);
    }
}
