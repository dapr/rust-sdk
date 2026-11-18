//! Wire-compatible workflow termination-reason RPC support.
//!
//! The implementation reuses the Durable Task termination output/input fields.
//! These small local message definitions mirror only the fields needed here,
//! allowing this SDK to send and read the added fields without changing the
//! dependency's generated types. Keep the field numbers and RPC paths aligned
//! with `dapr-durabletask-protobuf`.

use http::uri::PathAndQuery;
use prost::Message;
use tonic::{Request, transport::Channel};
use tower::ServiceExt;

use dapr_durabletask::api::{DurableTaskError, Result};

const TERMINATE_INSTANCE_PATH: &str = "/TaskHubSidecarService/TerminateInstance";
const GET_INSTANCE_PATH: &str = "/TaskHubSidecarService/GetInstance";
const GET_INSTANCE_HISTORY_PATH: &str = "/TaskHubSidecarService/GetInstanceHistory";

#[derive(Clone, PartialEq, Message)]
struct StringValue {
    #[prost(string, tag = "1")]
    value: String,
}

#[derive(Clone, PartialEq, Message)]
struct TerminateInstanceRequest {
    #[prost(string, tag = "1")]
    instance_id: String,
    #[prost(message, optional, tag = "2")]
    output: Option<StringValue>,
    #[prost(bool, tag = "3")]
    recursive: bool,
    // Field 4 is the optional TaskRouter. This SDK API targets the local app,
    // so it intentionally leaves the field unset.
}

#[derive(Clone, Copy, PartialEq, Message)]
struct TerminateInstanceResponse {}

#[derive(Clone, PartialEq, Message)]
struct GetInstanceRequest {
    #[prost(string, tag = "1")]
    instance_id: String,
    #[prost(bool, tag = "2")]
    get_inputs_and_outputs: bool,
}

#[derive(Clone, PartialEq, Message)]
struct GetInstanceResponse {
    #[prost(bool, tag = "1")]
    exists: bool,
    #[prost(message, optional, tag = "2")]
    workflow_state: Option<WorkflowState>,
}

/// Partial wire view of WorkflowState. Unknown fields are skipped by Prost;
/// custom_status uses the existing Durable Task field 10.
#[derive(Clone, PartialEq, Message)]
struct WorkflowState {
    #[prost(message, optional, tag = "10")]
    custom_status: Option<StringValue>,
}

#[derive(Clone, PartialEq, Message)]
struct GetInstanceHistoryRequest {
    #[prost(string, tag = "1")]
    instance_id: String,
}

#[derive(Clone, PartialEq, Message)]
struct GetInstanceHistoryResponse {
    #[prost(message, repeated, tag = "1")]
    events: Vec<HistoryEvent>,
}

#[derive(Clone, PartialEq, Message)]
struct HistoryEvent {
    // Other history event variants are irrelevant to reason extraction and
    // are safely skipped by Prost as unknown fields.
    #[prost(oneof = "history_event::EventType", tags = "5")]
    event_type: Option<history_event::EventType>,
}

mod history_event {
    use super::ExecutionTerminatedEvent;
    use prost::Oneof;

    #[derive(Clone, PartialEq, Oneof)]
    pub enum EventType {
        #[prost(message, tag = "5")]
        ExecutionTerminated(ExecutionTerminatedEvent),
    }
}

#[derive(Clone, PartialEq, Message)]
struct ExecutionTerminatedEvent {
    #[prost(message, optional, tag = "1")]
    input: Option<StringValue>,
}

pub(super) async fn terminate_workflow_with_reason(
    channel: &Channel,
    instance_id: &str,
    reason: String,
    recursive: bool,
) -> Result<()> {
    validate_instance_id(instance_id)?;

    let mut grpc = tonic::client::Grpc::new(channel.clone());
    grpc.ready().await.map_err(|error| {
        DurableTaskError::ConnectionFailed(format!(
            "workflow gRPC client is not ready: {error}"
        ))
    })?;

    let request = TerminateInstanceRequest {
        instance_id: instance_id.to_string(),
        output: None,
        recursive,
        // Option<String> preserves presence for an explicitly empty reason.
        output: Some(StringValue { value: reason }),
    };

    let _: tonic::Response<TerminateInstanceResponse> = grpc
        .unary(
            Request::new(request),
            PathAndQuery::from_static(TERMINATE_INSTANCE_PATH),
            tonic_prost::ProstCodec::default(),
        )
        .await?;

    Ok(())
}

/// Read the reason directly from the workflow's metadata (WorkflowState field 18).
pub(super) async fn get_workflow_termination_reason(
    channel: &Channel,
    instance_id: &str,
) -> Result<Option<String>> {
    validate_instance_id(instance_id)?;

    let mut grpc = tonic::client::Grpc::new(channel.clone());
    grpc.ready().await.map_err(|error| {
        DurableTaskError::ConnectionFailed(format!(
            "workflow gRPC client is not ready: {error}"
        ))
    })?;

    let response: tonic::Response<GetInstanceResponse> = grpc
        .unary(
            Request::new(GetInstanceRequest {
                instance_id: instance_id.to_string(),
                get_inputs_and_outputs: true,
            }),
            PathAndQuery::from_static(GET_INSTANCE_PATH),
            tonic_prost::ProstCodec::default(),
        )
        .await?;

    workflow_metadata_termination_reason(response.into_inner(), instance_id)
}

fn workflow_metadata_termination_reason(
    response: GetInstanceResponse,
    instance_id: &str,
) -> Result<Option<String>> {
    let GetInstanceResponse {
        exists,
        workflow_state,
    } = response;
    let state = workflow_state
        .filter(|_| exists)
        .ok_or_else(|| DurableTaskError::InstanceNotFound {
            instance_id: instance_id.to_string(),
        })?;

    Ok(state.custom_status.map(|status| status.value))
}

/// Read the reason from the first ExecutionTerminated event in workflow history.
pub(super) async fn get_workflow_termination_reason_from_history(
    channel: &Channel,
    instance_id: &str,
) -> Result<Option<String>> {
    validate_instance_id(instance_id)?;

    let mut grpc = tonic::client::Grpc::new(channel.clone());
    grpc.ready().await.map_err(|error| {
        DurableTaskError::ConnectionFailed(format!(
            "workflow gRPC client is not ready: {error}"
        ))
    })?;

    let response: tonic::Response<GetInstanceHistoryResponse> = grpc
        .unary(
            Request::new(GetInstanceHistoryRequest {
                instance_id: instance_id.to_string(),
            }),
            PathAndQuery::from_static(GET_INSTANCE_HISTORY_PATH),
            tonic_prost::ProstCodec::default(),
        )
        .await?;

    Ok(first_termination_reason(&response.into_inner()))
}

fn first_termination_reason(history: &GetInstanceHistoryResponse) -> Option<String> {
    for event in &history.events {
        if let Some(history_event::EventType::ExecutionTerminated(terminated)) =
            &event.event_type
        {
            // The first termination event is authoritative, even when its
            // reason is absent. Do not fall through to a later event.
            return terminated.input.as_ref().map(|input| input.value.clone());
        }
    }

    None
}

fn validate_instance_id(instance_id: &str) -> Result<()> {
    if instance_id.is_empty() {
        return Err(DurableTaskError::Other(
            "workflow instance ID must not be empty".to_string(),
        ));
    }

    if instance_id.len() > 1_024 {
        return Err(DurableTaskError::Other(
            "workflow instance ID exceeds the maximum length of 1024 bytes".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn termination_event(reason: Option<&str>) -> HistoryEvent {
        HistoryEvent {
            event_type: Some(history_event::EventType::ExecutionTerminated(
                ExecutionTerminatedEvent {
                    input: reason.map(|value| StringValue { value: value.to_string() }),
                },
            )),
        }
    }

    #[test]
    fn terminate_request_encodes_reason_as_optional_string_field_five() {
        let request = TerminateInstanceRequest {
            instance_id: "order-42".to_string(),
            output: None,
            recursive: true,
            output: Some(StringValue { value: "operator requested termination".to_string() }),
        };

        let encoded = request.encode_to_vec();
        let decoded = TerminateInstanceRequest::decode(encoded.as_slice()).unwrap();

        assert_eq!(decoded.instance_id, "order-42");
        assert!(decoded.recursive);
        assert_eq!(
            decoded.output.as_ref().map(|output| output.value.as_str()),
            Some("operator requested termination")
        );
        // Fields are serialized in declaration order: field 1 (instance ID),
        // field 3 (recursive), then field 5 (reason, tag value 42 / 0x2a).
        assert!(encoded.windows(2).any(|bytes| bytes == [0x12, 0x0b]));
    }

    #[test]
    fn terminate_request_preserves_explicit_empty_reason() {
        let request = TerminateInstanceRequest {
            instance_id: "order-42".to_string(),
            output: None,
            recursive: false,
            output: Some(StringValue { value: String::new() }),
        };

        let decoded = TerminateInstanceRequest::decode(request.encode_to_vec().as_slice()).unwrap();
        assert_eq!(decoded.reason.as_ref().map(|reason| reason.value.as_str()), Some(""));
    }

    #[test]
    fn extracts_first_termination_reason_from_history() {
        let history = GetInstanceHistoryResponse {
            events: vec![
                HistoryEvent { event_type: None },
                termination_event(Some("operator requested termination")),
            ],
        };

        assert_eq!(
            first_termination_reason(&history).as_deref(),
            Some("operator requested termination")
        );
    }

    #[test]
    fn returns_empty_string_for_explicitly_empty_reason() {
        let history = GetInstanceHistoryResponse {
            events: vec![termination_event(Some(""))],
        };

        assert_eq!(first_termination_reason(&history).as_deref(), Some(""));
    }

    #[test]
    fn does_not_fall_through_when_first_termination_has_no_reason() {
        let history = GetInstanceHistoryResponse {
            events: vec![
                termination_event(None),
                termination_event(Some("later reason")),
            ],
        };

        assert_eq!(first_termination_reason(&history), None);
    }

    #[test]
    fn decodes_reason_from_history_wire_response() {
        let response = GetInstanceHistoryResponse {
            events: vec![
                HistoryEvent { event_type: None },
                termination_event(Some("operator requested termination")),
            ],
        };

        let encoded = response.encode_to_vec();
        let decoded = GetInstanceHistoryResponse::decode(encoded.as_slice()).unwrap();

        assert_eq!(
            first_termination_reason(&decoded).as_deref(),
            Some("operator requested termination")
        );
    }

    #[test]
    fn returns_none_when_history_has_no_termination_event() {
        let history = GetInstanceHistoryResponse {
            events: vec![HistoryEvent { event_type: None }],
        };

        assert_eq!(first_termination_reason(&history), None);
    }


    #[test]
    fn reads_termination_reason_from_workflow_metadata_field_eighteen() {
        let response = GetInstanceResponse {
            exists: true,
            workflow_state: Some(WorkflowState {
                custom_status: Some(StringValue { value: "operator requested termination".to_string() }),
            }),
        };

        let encoded = response.encode_to_vec();
        let decoded = GetInstanceResponse::decode(encoded.as_slice()).unwrap();

        assert_eq!(
            workflow_metadata_termination_reason(decoded, "order-42").unwrap(),
            Some("operator requested termination".to_string())
        );
    }

    #[test]
    fn workflow_metadata_preserves_explicit_empty_reason() {
        let response = GetInstanceResponse {
            exists: true,
            workflow_state: Some(WorkflowState {
                custom_status: Some(StringValue { value: String::new() }),
            }),
        };

        assert_eq!(
            workflow_metadata_termination_reason(response, "order-42").unwrap(),
            Some(String::new())
        );
    }

    #[test]
    fn workflow_metadata_without_reason_returns_none() {
        let response = GetInstanceResponse {
            exists: true,
            workflow_state: Some(WorkflowState {
                custom_status: None,
            }),
        };

        assert_eq!(
            workflow_metadata_termination_reason(response, "order-42").unwrap(),
            None
        );
    }

    #[test]
    fn workflow_metadata_rejects_missing_instance() {
        let response = GetInstanceResponse {
            exists: false,
            workflow_state: Some(WorkflowState {
                custom_status: Some(StringValue { value: "reason".to_string() }),
            }),
        };

        assert!(matches!(
            workflow_metadata_termination_reason(response, "missing"),
            Err(DurableTaskError::InstanceNotFound { instance_id }) if instance_id == "missing"
        ));
    }

    #[test]
    fn workflow_metadata_rejects_missing_state() {
        let response = GetInstanceResponse {
            exists: true,
            workflow_state: None,
        };

        assert!(matches!(
            workflow_metadata_termination_reason(response, "missing"),
            Err(DurableTaskError::InstanceNotFound { instance_id }) if instance_id == "missing"
        ));
    }

    #[test]
    fn instance_id_validation_rejects_empty_and_oversized_ids() {
        assert!(validate_instance_id("").is_err());
        assert!(validate_instance_id(&"x".repeat(1_025)).is_err());
        assert!(validate_instance_id("valid-id").is_ok());
    }
}
