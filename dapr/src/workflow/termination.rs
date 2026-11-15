//! Wire-compatible workflow termination-reason RPC support.
//!
//! The currently published `dapr-durabletask` protobuf crate predates the
//! `TerminateRequest.reason` and `ExecutionTerminatedEvent.reason` fields.
//! These small local message definitions mirror only the fields needed here,
//! allowing this SDK to send and read the added fields without changing the
//! dependency's generated types. Keep the field numbers and RPC paths aligned
//! with `dapr-durabletask-protobuf`.

use prost::Message;
use tonic::{Request, codegen::http::uri::PathAndQuery, transport::Channel};
use tower::ServiceExt;

use dapr_durabletask::api::{DurableTaskError, Result};

const TERMINATE_INSTANCE_PATH: &str = "/TaskHubSidecarService/TerminateInstance";
const GET_INSTANCE_HISTORY_PATH: &str = "/TaskHubSidecarService/GetInstanceHistory";

#[derive(Clone, PartialEq, Message)]
struct TerminateInstanceRequest {
    #[prost(string, tag = "1")]
    instance_id: String,
    #[prost(message, optional, tag = "2")]
    output: Option<String>,
    #[prost(bool, tag = "3")]
    recursive: bool,
    // Field 4 is the optional TaskRouter. This SDK API targets the local app,
    // so it intentionally leaves the field unset.
    #[prost(message, optional, tag = "5")]
    reason: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Message)]
struct TerminateInstanceResponse {}

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
    #[prost(int32, tag = "1")]
    event_id: i32,
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
    // Fields 1 and 2 (input and recurse) are intentionally omitted.
    #[prost(message, optional, tag = "3")]
    reason: Option<String>,
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
        reason: Some(reason),
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
    history.events.iter().find_map(|event| match &event.event_type {
        Some(history_event::EventType::ExecutionTerminated(terminated)) => {
            // The first termination event is authoritative. Returning its
            // optional reason (including Some("")) avoids accidentally
            // reporting a later event's reason for malformed/replayed history.
            Some(terminated.reason.clone())
        }
        _ => None,
    })?
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

    fn termination_event(event_id: i32, reason: Option<&str>) -> HistoryEvent {
        HistoryEvent {
            event_id,
            event_type: Some(history_event::EventType::ExecutionTerminated(
                ExecutionTerminatedEvent {
                    reason: reason.map(str::to_string),
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
            reason: Some("operator requested termination".to_string()),
        };

        let encoded = request.encode_to_vec();
        let decoded = TerminateInstanceRequest::decode(encoded.as_slice()).unwrap();

        assert_eq!(decoded.instance_id, "order-42");
        assert!(decoded.recursive);
        assert_eq!(
            decoded.reason.as_deref(),
            Some("operator requested termination")
        );
        // The reason field is a length-delimited protobuf field numbered 5
        // (tag value 42 / 0x2a).
        assert!(encoded.contains(&0x2a));
    }

    #[test]
    fn terminate_request_preserves_explicit_empty_reason() {
        let request = TerminateInstanceRequest {
            instance_id: "order-42".to_string(),
            output: None,
            recursive: false,
            reason: Some(String::new()),
        };

        let decoded = TerminateInstanceRequest::decode(request.encode_to_vec().as_slice()).unwrap();
        assert_eq!(decoded.reason.as_deref(), Some(""));
    }

    #[test]
    fn extracts_first_termination_reason_from_history() {
        let history = GetInstanceHistoryResponse {
            events: vec![
                HistoryEvent {
                    event_id: 1,
                    event_type: None,
                },
                termination_event(2, Some("operator requested termination")),
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
            events: vec![termination_event(1, Some(""))],
        };

        assert_eq!(first_termination_reason(&history).as_deref(), Some(""));
    }

    #[test]
    fn does_not_fall_through_when_first_termination_has_no_reason() {
        let history = GetInstanceHistoryResponse {
            events: vec![
                termination_event(1, None),
                termination_event(2, Some("later reason")),
            ],
        };

        assert_eq!(first_termination_reason(&history), None);
    }

    #[test]
    fn returns_none_when_history_has_no_termination_event() {
        let history = GetInstanceHistoryResponse {
            events: vec![HistoryEvent {
                event_id: 1,
                event_type: None,
            }],
        };

        assert_eq!(first_termination_reason(&history), None);
    }

    #[test]
    fn instance_id_validation_rejects_empty_and_oversized_ids() {
        assert!(validate_instance_id("").is_err());
        assert!(validate_instance_id(&"x".repeat(1_025)).is_err());
        assert!(validate_instance_id("valid-id").is_ok());
    }
}
