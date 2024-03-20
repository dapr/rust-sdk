use futures::StreamExt;
use tonic::{Request, Status};
use tonic::codegen::tokio_stream;

use crate::Client;
use crate::client::TonicClient;
use crate::dapr::dapr::proto::common::v1::StreamPayload;
use crate::dapr::dapr::proto::runtime::v1::{
    DecryptRequest, DecryptRequestOptions, EncryptRequest, EncryptRequestOptions,
};

impl Client<TonicClient> {
    pub async fn encrypt<T>(
        &mut self,
        payload: &T,
        request_options: EncryptRequestOptions,
    ) -> Result<Vec<StreamPayload>, Status>
    where
        T: Into<Vec<u8>> + Clone,
    {
        let stream_payload = StreamPayload {
            data: payload.clone().into(),
            seq: 0,
        };
        let request = EncryptRequest {
            options: Some(request_options),
            payload: Some(stream_payload),
        };
        let request = Request::new(tokio_stream::iter([request]));
        let stream = self.0.encrypt_alpha1(request).await?;
        let mut stream = stream.into_inner();
        let mut return_data = vec![];
        while let Some(resp) = stream.next().await {
            if let Ok(resp) = resp {
                if let Some(data) = resp.payload {
                    return_data.push(data)
                }
            }
        }
        Ok(return_data)
    }

    pub async fn decrypt(
        &mut self,
        encrypted: Vec<StreamPayload>,
        options: DecryptRequestOptions,
    ) -> Result<Vec<u8>, Status> {
        let requested_items: Vec<DecryptRequest> = encrypted
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if i == 0 {
                    DecryptRequest {
                        options: Some(options.clone()),
                        payload: Some(item.clone()),
                    }
                } else {
                    DecryptRequest {
                        options: None,
                        payload: Some(item.clone()),
                    }
                }
            })
            .collect();
        let request = Request::new(tokio_stream::iter(requested_items));
        let stream = self.0.decrypt_alpha1(request).await?;
        let mut stream = stream.into_inner();
        let mut data = vec![];
        while let Some(resp) = stream.next().await {
            if let Ok(resp) = resp {
                if let Some(mut payload) = resp.payload {
                    data.append(payload.data.as_mut())
                }
            }
        }
        Ok(data)
    }
}
