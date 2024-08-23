use std::time::Duration;

use tonic::transport::{Channel, Uri};

use crate::dapr::proto::{common::v1::*, runtime::v1::dapr_client::DaprClient, runtime::v1::*};
use crate::error::Error;

/// Dapr env var constants
//const DAPR_GRPC_ENDPOINT_ENV_VAR_NAME:  &str = "DAPR_GRPC_ENDPOINT";
//const DAPR_API_MAX_RETRIES_ENV_VAR_NAME: &str = "DAPR_API_MAX_RETRIES";
//const DAPR_API_TIMEOUT_SECONDS_ENV_VAR_NAME: &str = "DAPR_API_TIMEOUT_SECONDS";

/// Client implementation for interfacing with Dapr
#[derive(Clone, Debug)]
pub struct Client {
    client: DaprClient<Channel>,
}

impl Client {
    async fn new_internal(addr: Uri, keep_alive: Option<Duration>) -> Result<Self, Error> {
        let builder = Channel::builder(addr)
            .connect_timeout(Duration::from_secs(60))
            .keep_alive_timeout(keep_alive.unwrap_or_else(|| Duration::from_secs(60)));

        // Create a channel that connects immediately or errors
        let channel = builder.connect().await?;

        let dapr_client = DaprClient::new(channel);
        Ok(Self {
            client: dapr_client,
        })
    }

    /// Create a dapr instance with the endpoint from env var or default
    pub async fn new() -> Result<Self, Error> {
        let addr: Uri = format!("{}:{}", "http://localhost", "50051")
            .parse()
            .unwrap();
        let keep_alive = Some(Duration::from_secs(60));
        Self::new_internal(addr, keep_alive).await
        // TODO: Cleanup implementation
    }

    /// Invokes a method on a remote Dapr app.
    pub async fn invoke_service(
        &mut self,
        _req: InvokeServiceRequest,
    ) -> Result<InvokeResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Gets the state for a specific key.
    pub async fn get_state(&mut self, _req: GetStateRequest) -> Result<GetStateResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Gets the state of each key in a list.
    pub async fn get_bulk_state(
        &mut self,
        _req: GetBulkStateRequest,
    ) -> Result<GetBulkStateResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Saves the state for a specific key.
    pub async fn save_state(&mut self, _req: SaveStateRequest) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Queries the state.
    pub async fn query_state_alpha1(
        &mut self,
        _req: QueryStateRequest,
    ) -> Result<QueryStateResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Deletes the state for a specific key.
    pub async fn delete_state(&mut self, _req: DeleteStateRequest) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Deletes the state for a list of keys.
    pub async fn delete_bulk_state(&mut self, _req: DeleteBulkStateRequest) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Executes a transaction for a specific store.
    pub async fn execute_state_transaction(
        &mut self,
        _req: ExecuteStateTransactionRequest,
    ) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Publishes an event to the specified topic.
    pub async fn publish_event(&mut self, _req: PublishEventRequest) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Publishes multiple events to the specified topic.
    pub async fn bulk_publish_event_alpha1(
        &mut self,
        _req: BulkPublishRequest,
    ) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Subscribes to a pubsub topic and creates a stream to receive events.
    pub async fn subscribe_topic_events_alpha1(&mut self) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Invokes a binding.
    pub async fn invoke_binding(
        &mut self,
        _req: InvokeBindingRequest,
    ) -> Result<InvokeBindingResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Gets a secret from a secrets store.
    pub async fn get_secret(&mut self, _req: GetSecretRequest) -> Result<GetSecretResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Gets multiple secrets from a secrets store.
    pub async fn get_bulk_secret(
        &mut self,
        _req: GetBulkStateRequest,
    ) -> Result<GetBulkSecretResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Register an actor timer.
    pub async fn register_actor_timer(
        &mut self,
        _req: RegisterActorTimerRequest,
    ) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Unregister an actor timer.
    pub async fn unregister_actor_timer(
        &mut self,
        _req: UnregisterActorTimerRequest,
    ) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Register an actor reminder.
    pub async fn register_actor_reminder(
        &mut self,
        _req: RegisterActorReminderRequest,
    ) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Unregister an actor reminder.
    pub async fn unregister_actor_reminder(
        &mut self,
        _req: UnregisterActorReminderRequest,
    ) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Gets the state of a specified actor.
    pub async fn get_actor_state(
        &mut self,
        _req: GetActorStateRequest,
    ) -> Result<GetActorStateResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Executes state transaction for a specified actor.
    pub async fn execute_actor_state_transaction(
        &mut self,
        _req: ExecuteActorStateTransactionRequest,
    ) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Calls a method on the specified actor.
    pub async fn invoke_actor(
        &mut self,
        _req: InvokeActorRequest,
    ) -> Result<InvokeActorResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Gets a configuration from the configuration store.
    #[deprecated(note = "use `get_configuration` instead")]
    pub async fn get_configuration_alpha1(
        &mut self,
        req: GetConfigurationRequest,
    ) -> Result<GetConfigurationResponse, Error> {
        self.get_configuration(req).await
    }

    /// Gets a configuration from the configuration store.
    pub async fn get_configuration(
        &mut self,
        _req: GetConfigurationRequest,
    ) -> Result<GetConfigurationResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Subscribes to a configuration item and creates a stream to receive updates.
    #[deprecated(note = "use `subscribe_to_configuration` instead")]
    pub async fn subscribe_to_configuration_alpha1(
        &mut self,
        req: SubscribeConfigurationRequest,
    ) -> Result<(), Error> {
        self.subscribe_to_configuration(req).await
    }

    /// Subscribes to a configuration item and creates a stream to receive updates.
    pub async fn subscribe_to_configuration(
        &mut self,
        _req: SubscribeConfigurationRequest,
    ) -> Result<(), Error> {
        // return an empty stream and an error
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Unsubscribes from a configuration item subscription.
    #[deprecated(note = "use `unsubscribe_from_configuration` instead")]
    pub async fn unsubscribe_from_configuration_alpha1(
        &mut self,
        req: UnsubscribeConfigurationRequest,
    ) -> Result<UnsubscribeConfigurationResponse, Error> {
        self.unsubscribe_from_configuration(req).await
    }

    /// Unsubscribes from a configuration item subscription.
    pub async fn unsubscribe_from_configuration(
        &mut self,
        _req: UnsubscribeConfigurationRequest,
    ) -> Result<UnsubscribeConfigurationResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Tries to get a lock with an expiry.
    pub async fn try_lock_alpha1(
        &mut self,
        _req: TryLockRequest,
    ) -> Result<TryLockResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Unlocks a lock.
    pub async fn unlock_alpha1(&mut self, _req: UnlockRequest) -> Result<UnlockResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Encrypts a message using the Dapr encryption scheme and a key stored in the vault.
    pub async fn encrypt_alpha1(&mut self, _req: EncryptRequest) -> Result<EncryptResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Decrypts a message using the Dapr encryption scheme and a key stored in the vault.
    pub async fn decrypt_alpha1(&mut self, _req: DecryptRequest) -> Result<DecryptResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Gets the sidecar metadata.
    pub async fn get_metadata(&mut self) -> Result<GetMetadataResponse, Error> {
        let request: GetMetadataRequest = GetMetadataRequest {};
        Ok(self
            .client
            .get_metadata(request)
            .await
            .unwrap()
            .into_inner())
    }

    /// Appends a key-value pair to the sidecar metadata.
    pub async fn set_metadata(&mut self, _req: SetMetadataRequest) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Retrieves a public key from the vault.
    pub async fn subtle_get_key_alpha1(
        &mut self,
        _req: SubtleGetKeyRequest,
    ) -> Result<SubtleGetKeyResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Encrypts a small message using a key stored in the vault.
    pub async fn subtle_encrypt_alpha1(
        &mut self,
        _req: SubtleEncryptRequest,
    ) -> Result<SubtleEncryptResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Decrypts a small message using a key stored in the vault.
    pub async fn subtle_decrypt_alpha1(
        &mut self,
        _req: SubtleDecryptRequest,
    ) -> Result<SubtleDecryptResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Wraps a key using a key stored in the vault.
    pub async fn subtle_wrap_key_alpha1(
        &mut self,
        _req: SubtleWrapKeyRequest,
    ) -> Result<SubtleWrapKeyResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Unwraps a key using a key stored in the vault.
    pub async fn subtle_unwrap_key_alpha1(
        &mut self,
        _req: SubtleUnwrapKeyRequest,
    ) -> Result<SubtleUnwrapKeyResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Signs a message using a key stored in the vault.
    pub async fn subtle_sign_alpha1(
        &mut self,
        _req: SubtleSignRequest,
    ) -> Result<SubtleSignResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// Verifies a message signature using a key stored in the vault.
    pub async fn subtle_verify_alpha1(
        &mut self,
        _req: SubtleVerifyRequest,
    ) -> Result<SubtleVerifyResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Starts a workflow instance.
    #[deprecated(note = "use `start_workflow_beta1` instead")]
    pub async fn start_workflow_alpha1(
        &mut self,
        req: StartWorkflowRequest,
    ) -> Result<StartWorkflowResponse, Error> {
        self.start_workflow_beta1(req).await
    }

    /// (Beta) Starts a workflow instance.
    pub async fn start_workflow_beta1(
        &mut self,
        _req: StartWorkflowRequest,
    ) -> Result<StartWorkflowResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Gets a workflow instance.
    #[deprecated(note = "use `get_workflow_beta1` instead")]
    pub async fn get_workflow_alpha1(
        &mut self,
        req: GetWorkflowRequest,
    ) -> Result<GetWorkflowResponse, Error> {
        self.get_workflow_beta1(req).await
    }

    /// (Beta) Gets a workflow instance.
    pub async fn get_workflow_beta1(
        &mut self,
        _req: GetWorkflowRequest,
    ) -> Result<GetWorkflowResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Purges the workflow instance.
    #[deprecated(note = "use `purge_workflow_beta1` instead")]
    pub async fn purge_workflow_alpha1(&mut self, req: PurgeWorkflowRequest) -> Result<(), Error> {
        self.purge_workflow_beta1(req).await
    }

    /// (Beta) Purges the workflow instance.

    pub async fn purge_workflow_beta1(&mut self, _req: PurgeWorkflowRequest) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Terminates a running workflow instance.
    #[deprecated(note = "use `terminate_workflow_beta1` instead")]
    pub async fn terminate_workflow_alpha1(
        &mut self,
        req: TerminateWorkflowRequest,
    ) -> Result<(), Error> {
        self.terminate_workflow_beta1(req).await
    }

    /// (Beta) Terminates a running workflow instance.
    pub async fn terminate_workflow_beta1(
        &mut self,
        _req: TerminateWorkflowRequest,
    ) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Pauses a running workflow instance.

    #[deprecated(note = "use `pause_workflow_beta1` instead")]
    pub async fn pause_workflow_alpha1(&mut self, req: PauseWorkflowRequest) -> Result<(), Error> {
        self.pause_workflow_beta1(req).await
    }

    /// (Beta) Pauses a running workflow instance.
    pub async fn pause_workflow_beta1(&mut self, _req: PauseWorkflowRequest) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Resumes a paused workflow instance.
    #[deprecated(note = "use `resume_workflow_beta1` instead")]
    pub async fn resume_workflow_alpha1(
        &mut self,
        req: ResumeWorkflowRequest,
    ) -> Result<(), Error> {
        self.resume_workflow_beta1(req).await
    }

    /// (Beta) Resumes a paused workflow instance.
    pub async fn resume_workflow_beta1(
        &mut self,
        _req: ResumeWorkflowRequest,
    ) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Raises an event on a running workflow instance.
    #[deprecated(note = "use `raise_event_workflow_beta1` instead")]
    pub async fn raise_event_workflow_alpha1(
        &mut self,
        req: RaiseEventWorkflowRequest,
    ) -> Result<(), Error> {
        self.raise_event_workflow_beta1(req).await
    }

    /// (Beta) Raises an event on a running workflow instance.
    pub async fn raise_event_workflow_beta1(
        &mut self,
        _req: RaiseEventWorkflowRequest,
    ) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// Requests the sidecar to shut down gracefully.
    pub async fn shutdown(&mut self, _req: ShutdownRequest) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Schedules a job.
    pub async fn schedule_job_alpha1(
        &mut self,
        _req: ScheduleJobRequest,
    ) -> Result<ScheduleJobResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Retrieves a scheduled job.
    pub async fn get_job_alpha1(&mut self, _req: GetJobRequest) -> Result<GetJobResponse, Error> {
        Err(Error::UnimplementedError)
    }

    /// (Alpha) Deletes a scheduled job.
    pub async fn delete_job_alpha1(&mut self, _req: DeleteJobRequest) -> Result<(), Error> {
        Err(Error::UnimplementedError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new() {
        let client_resp = Client::new().await;
        assert!(client_resp.is_ok());
    }
}
