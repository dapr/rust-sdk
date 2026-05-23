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
    fn fetch_options_with_fetch_payloads_toggles() {
        let opts = FetchOptions::new();
        assert!(!opts.fetch_payloads);
        let opts = opts.with_fetch_payloads(true);
        assert!(opts.fetch_payloads);
    }
}
