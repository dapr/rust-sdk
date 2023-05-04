use std::{time::Duration};
use async_trait::async_trait;
use prost_types::Any;
use tonic::{transport::Channel as TonicChannel, Request};
use crate::dapr::dapr::proto::{runtime::v1 as dapr_v1};
use crate::error::Error as DaprError;

pub type GrpcDaprClient = dapr_v1::dapr_client::DaprClient<TonicChannel>;

pub enum ActorStateOperation {
  Upsert {
      key: String,
      value: Option<Vec<u8>>,
  },
  Delete {
      key: String,
  },
}

impl Into<TransactionalActorStateOperation> for ActorStateOperation {
  fn into(self) -> TransactionalActorStateOperation {
      match self {
          
          ActorStateOperation::Upsert { key, value } => TransactionalActorStateOperation {
              operation_type: "upsert".to_string(),
              key: key,
              value: match value {
                  Some(v) => Some(Any {
                      type_url: "type.googleapis.com/bytes".to_string(),
                      value: v,
                  }),
                  None => None,
              },
          },
          ActorStateOperation::Delete { key} => TransactionalActorStateOperation {
              operation_type: "delete".to_string(),
              key: key,
              value: None,
          },
      }
  }
}

#[async_trait]
pub trait DaprActorInterface {
  async fn connect(addr: String) -> Result<Self, DaprError> where Self: Sized;
  async fn get_actor_state(&mut self, request: GetActorStateRequest) -> Result<GetActorStateResponse, DaprError>;
  async fn register_actor_reminder(&mut self, request: RegisterActorReminderRequest) -> Result<(), DaprError>;
  async fn register_actor_timer(&mut self, request: RegisterActorTimerRequest) -> Result<(), DaprError>;
  async fn unregister_actor_reminder(&mut self, request: UnregisterActorReminderRequest) -> Result<(), DaprError>;
  async fn unregister_actor_timer(&mut self, request: UnregisterActorTimerRequest) -> Result<(), DaprError>;
  async fn execute_actor_state_transaction(&mut self, request: ExecuteActorStateTransactionRequest) -> Result<(), DaprError>;
}

#[async_trait]
impl DaprActorInterface for dapr_v1::dapr_client::DaprClient<TonicChannel> {
  async fn connect(addr: String) -> Result<Self, DaprError> {
      Ok(dapr_v1::dapr_client::DaprClient::connect(addr).await?)
  }
  async fn get_actor_state(&mut self, request: GetActorStateRequest) -> Result<GetActorStateResponse, DaprError> {
      Ok(self.get_actor_state(Request::new(request)).await?.into_inner())
  }

  async fn register_actor_reminder(&mut self, request: RegisterActorReminderRequest) -> Result<(), DaprError> {
      Ok(self.register_actor_reminder(Request::new(request)).await?.into_inner())
  }

  async fn register_actor_timer(&mut self, request: RegisterActorTimerRequest) -> Result<(), DaprError> {
      Ok(self.register_actor_timer(Request::new(request)).await?.into_inner())
  }

  async fn unregister_actor_reminder(&mut self, request: UnregisterActorReminderRequest) -> Result<(), DaprError> {
      Ok(self.unregister_actor_reminder(Request::new(request)).await?.into_inner())
  }

  async fn unregister_actor_timer(&mut self, request: UnregisterActorTimerRequest) -> Result<(), DaprError> {
      Ok(self.unregister_actor_timer(Request::new(request)).await?.into_inner())
  }

  async fn execute_actor_state_transaction(&mut self, request: ExecuteActorStateTransactionRequest) -> Result<(), DaprError> {
      Ok(self.execute_actor_state_transaction(Request::new(request)).await?.into_inner())
  }
}

pub struct ActorContextClient<T>{
  client: T,
  actor_type: String,
  actor_id: String,
}

impl<T: DaprActorInterface> ActorContextClient<T> {
  
  pub fn new(client: T, actor_type: &str, actor_id: &str) -> Self {
      ActorContextClient{
          client,
          actor_type: actor_type.to_string(),
          actor_id: actor_id.to_string(),
      }
  }

  pub fn get_actor_state<K>(&mut self, key: K) -> Result<GetActorStateResponse, DaprError>
  where K: Into<String>
  {
      futures::executor::block_on(
        self.client
            .get_actor_state(GetActorStateRequest {
                actor_type: self.actor_type.to_string(),
                actor_id: self.actor_id.to_string(),
                key: key.into(),
            })
        )
  }

  pub fn execute_actor_state_transaction(
      &mut self,
      operations: Vec<ActorStateOperation>,
  ) -> Result<(), DaprError>
  {
    futures::executor::block_on(self.client
          .execute_actor_state_transaction(ExecuteActorStateTransactionRequest {
              actor_type: self.actor_type.to_string(),
              actor_id: self.actor_id.to_string(),
              operations: operations.into_iter().map(|o| o.into()).collect(),
          }))
  }

  pub fn register_actor_reminder<I>(
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
    futures::executor::block_on(self.client
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
              data: data,
              ttl: match ttl {
                  None => "".to_string(),
                  Some(t) => chrono::Duration::from_std(t).unwrap().to_string(),
              },
              
          }))
  }

  pub fn unregister_actor_reminder<I>(
      &mut self,
      name: I
  ) -> Result<(), DaprError>
  where
      I: Into<String>,
  {
    futures::executor::block_on(self.client
          .unregister_actor_reminder(UnregisterActorReminderRequest {
              actor_type: self.actor_type.to_string(),
              actor_id: self.actor_id.to_string(),
              name: name.into(),
          }))
  }

  pub fn register_actor_timer<I>(
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
    futures::executor::block_on(self.client
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
              data: data,
              callback: callback.unwrap_or_default(),
              ttl: match ttl {
                  None => "".to_string(),
                  Some(t) => chrono::Duration::from_std(t).unwrap().to_string(),
              },
          }))
  }

  pub fn unregister_actor_timer<I>(
      &mut self,
      name: I
  ) -> Result<(), DaprError>
  where
      I: Into<String>,
  {
    futures::executor::block_on(self.client
          .unregister_actor_timer(UnregisterActorTimerRequest {
              actor_type: self.actor_type.to_string(),
              actor_id: self.actor_id.to_string(),
              name: name.into(),
          }))
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