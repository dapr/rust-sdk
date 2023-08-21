use crate::client::TonicClient;
use crate::dapr::dapr::proto::runtime::v1 as dapr_v1;
use crate::error::Error as DaprError;
use prost_types::Any;
use std::collections::HashMap;
use std::time::Duration;
use tonic::transport::Channel as TonicChannel;

pub type GrpcDaprClient = dapr_v1::dapr_client::DaprClient<TonicChannel>;

pub enum ActorStateOperation {
    Upsert { key: String, value: Option<Vec<u8>> },
    Delete { key: String },
}

impl From<ActorStateOperation> for TransactionalActorStateOperation {
    fn from(val: ActorStateOperation) -> Self {
        match val {
            ActorStateOperation::Upsert { key, value } => TransactionalActorStateOperation {
                operation_type: "upsert".to_string(),
                key,
                value: value.map(|v| Any {
                        type_url: "type.googleapis.com/bytes".to_string(),
                        value: v,
                    }),
                metadata: HashMap::new(),
            },
            ActorStateOperation::Delete { key } => TransactionalActorStateOperation {
                operation_type: "delete".to_string(),
                key,
                value: None,
                metadata: HashMap::new(),
            },
        }
    }
}

/// A client for interacting with the Dapr runtime within the scope of an actor.
///
/// Hosts methods for interacting with the Dapr sidecar specific to the actor instance.
#[derive(Clone)]
pub struct ActorContextClient {
    client: TonicClient,
    actor_type: String,
    actor_id: String,
}

impl ActorContextClient {
    pub fn new(client: TonicClient, actor_type: &str, actor_id: &str) -> Self {
        ActorContextClient {
            client,
            actor_type: actor_type.to_string(),
            actor_id: actor_id.to_string(),
        }
    }

    /// Retrieves a keyed state value within the scope of this instance of the actor.
    ///
    /// # Arguments
    /// * `key` - The key of the state to retrieve.
    pub async fn get_actor_state<K>(&mut self, key: K) -> Result<GetActorStateResponse, DaprError>
    where
        K: Into<String>,
    {
        Ok(self
            .client
            .get_actor_state(GetActorStateRequest {
                actor_type: self.actor_type.to_string(),
                actor_id: self.actor_id.to_string(),
                key: key.into(),
            })
            .await?
            .into_inner())
    }

    /// Saves a state value within the scope of this instance of the actor.
    ///
    /// # Arguments
    /// * `operations` - A list of [ActorStateOperation] to perform on the state.
    pub async fn execute_actor_state_transaction(
        &mut self,
        operations: Vec<ActorStateOperation>,
    ) -> Result<(), DaprError> {
        self
        .client
        .execute_actor_state_transaction(ExecuteActorStateTransactionRequest {
            actor_type: self.actor_type.to_string(),
            actor_id: self.actor_id.to_string(),
            operations: operations.into_iter().map(|o| o.into()).collect(),
        })
        .await?
        .into_inner();
        Ok(())
    }

    /// Registers a reminder with the Dapr runtime.
    ///
    /// # Arguments
    /// * `name` - The name of the reminder.
    /// * `due_time` - The time at which the reminder should first be invoked.
    /// * `period` - The time interval between invocations of the reminder.
    /// * `data` - The data to pass to the reminder when it is invoked.
    /// * `ttl` - The time to live for the reminder.
    pub async fn register_actor_reminder<I>(
        &mut self,
        name: I,
        due_time: Option<Duration>,
        period: Option<Duration>,
        data: Vec<u8>,
        ttl: Option<Duration>,
    ) -> Result<(), DaprError>
    where
        I: Into<String>,
    {
        self
        .client
        .register_actor_reminder(RegisterActorReminderRequest {
            actor_type: self.actor_type.to_string(),
            actor_id: self.actor_id.to_string(),
            name: name.into(),
            due_time: match due_time {
                None => "".to_string(),
                Some(t) => chrono::Duration::from_std(t).unwrap().to_string(),
            },
            period: match period {
                None => "".to_string(),
                Some(t) => chrono::Duration::from_std(t).unwrap().to_string(),
            },
            data,
            ttl: match ttl {
                None => "".to_string(),
                Some(t) => chrono::Duration::from_std(t).unwrap().to_string(),
            },
        })
        .await?
        .into_inner();
        Ok(())
    }

    /// Unregisters a reminder with the Dapr runtime.
    ///
    /// # Arguments
    /// * `name` - The name of the reminder to unregister.
    pub async fn unregister_actor_reminder<I>(&mut self, name: I) -> Result<(), DaprError>
    where
        I: Into<String>,
    {
        self
        .client
        .unregister_actor_reminder(UnregisterActorReminderRequest {
            actor_type: self.actor_type.to_string(),
            actor_id: self.actor_id.to_string(),
            name: name.into(),
        })
        .await?
        .into_inner();
        Ok(())
    }

    /// Registers a timer with the Dapr runtime.
    ///
    /// # Arguments
    /// * `name` - The name of the timer.
    /// * `due_time` - The time at which the timer should first be invoked.
    /// * `period` - The time interval between invocations of the timer.
    /// * `data` - The data to pass to the timer when it is invoked.
    /// * `callback` - The callback name to include in the invocation.
    /// * `ttl` - The time to live for the timer.
    pub async fn register_actor_timer<I>(
        &mut self,
        name: I,
        due_time: Option<Duration>,
        period: Option<Duration>,
        data: Vec<u8>,
        callback: Option<String>,
        ttl: Option<Duration>,
    ) -> Result<(), DaprError>
    where
        I: Into<String>,
    {
        self
        .client
        .register_actor_timer(RegisterActorTimerRequest {
            actor_type: self.actor_type.to_string(),
            actor_id: self.actor_id.to_string(),
            name: name.into(),
            due_time: match due_time {
                None => "".to_string(),
                Some(t) => chrono::Duration::from_std(t).unwrap().to_string(),
            },
            period: match period {
                None => "".to_string(),
                Some(t) => chrono::Duration::from_std(t).unwrap().to_string(),
            },
            data,
            callback: callback.unwrap_or_default(),
            ttl: match ttl {
                None => "".to_string(),
                Some(t) => chrono::Duration::from_std(t).unwrap().to_string(),
            },
        })
        .await?
        .into_inner();
        Ok(())
    }

    /// Unregisters a timer with the Dapr runtime.
    ///
    /// # Arguments
    /// * `name` - The name of the timer to unregister.
    pub async fn unregister_actor_timer<I>(&mut self, name: I) -> Result<(), DaprError>
    where
        I: Into<String>,
    {
        self
        .client
        .unregister_actor_timer(UnregisterActorTimerRequest {
            actor_type: self.actor_type.to_string(),
            actor_id: self.actor_id.to_string(),
            name: name.into(),
        })
        .await?
        .into_inner();
        Ok(())
    }
}

pub type GetActorStateRequest = dapr_v1::GetActorStateRequest;

pub type GetActorStateResponse = dapr_v1::GetActorStateResponse;

pub type ExecuteActorStateTransactionRequest = dapr_v1::ExecuteActorStateTransactionRequest;

pub type TransactionalActorStateOperation = dapr_v1::TransactionalActorStateOperation;

pub type RegisterActorTimerRequest = dapr_v1::RegisterActorTimerRequest;

pub type RegisterActorReminderRequest = dapr_v1::RegisterActorReminderRequest;

pub type UnregisterActorTimerRequest = dapr_v1::UnregisterActorTimerRequest;

pub type UnregisterActorReminderRequest = dapr_v1::UnregisterActorReminderRequest;
