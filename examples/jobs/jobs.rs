use std::time::Duration;

use base64::prelude::*;
use prost_types::Any;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tonic::transport::Server;
use tonic::Status;

use dapr::client::JobBuilder;
use dapr::dapr::dapr::proto::runtime::v1::app_callback_alpha_server::AppCallbackAlphaServer;
use dapr::dapr::dapr::proto::runtime::v1::{JobEventRequest, JobEventResponse};
use dapr::server::appcallbackalpha::{AppCallbackServiceAlpha, JobHandlerMethod};

type DaprClient = dapr::Client<dapr::client::TonicClient>;

#[derive(Serialize, Deserialize, Debug)]
struct Backup {
    task: String,
    metadata: Option<Metadata>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    db_name: String,
    backup_location: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonAny {
    type_url: String,
    value: String,
}
async fn backup_job_handler(request: JobEventRequest) -> Result<JobEventResponse, Status> {
    // Implement the logic for handling the backup job request
    // ...
    println!("received job");

    let mut data = Backup {
        task: "".to_string(),
        metadata: Some(Metadata {
            db_name: "".to_string(),
            backup_location: "".to_string(),
        }),
    };
    if request.data.is_some() {
        // weird value - any type is actually put into the value
        let any = request.data.unwrap().value;

        // parse any value
        let any_parsed: JsonAny = serde_json::from_slice(&any).unwrap();

        // Decode the base64-encoded value field
        let decoded_value = BASE64_STANDARD.decode(any_parsed.value).unwrap();

        // Deserialize the decoded value into a Backup struct
        let backup_val: Backup = serde_json::from_slice(&decoded_value).unwrap();

        println!("backup_val: {:?}", backup_val);

        data = backup_val;
    }

    println!(
        "name: {}, data: {:?}, method: {}, contenttype: {}, httpextension: {:?}",
        request.name, data, request.method, request.content_type, request.http_extension
    );

    Ok(JobEventResponse::default())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(async move {
        let server_addr = "127.0.0.1:50001".parse().unwrap();

        println!("AppCallbackAlpha server listening on {server_addr}");

        let mut alpha_callback_service = AppCallbackServiceAlpha::new();

        pub struct BackupHandler {}

        #[async_trait::async_trait]
        impl JobHandlerMethod for BackupHandler {
            async fn handler(&self, request: JobEventRequest) -> Result<JobEventResponse, Status> {
                backup_job_handler(request).await
            }
        }

        impl BackupHandler {
            pub fn new() -> Self {
                BackupHandler {}
            }
        }

        let handler_name = "prod-db-backup".to_string();

        alpha_callback_service.add_job_handler(handler_name, Box::new(BackupHandler::new()));

        Server::builder()
            .add_service(AppCallbackAlphaServer::new(alpha_callback_service))
            .serve(server_addr)
            .await
            .unwrap();
    });

    sleep(Duration::from_secs(5)).await;

    // Client

    let client_addr = "https://127.0.0.1".to_string();

    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let address = format!("{}:{}", client_addr, port);

    println!("attempting to create a dapr client: {}", address);

    // Create the client
    let mut client = DaprClient::connect(client_addr).await?;

    println!("client created");

    // define a job data in json
    let job = Backup {
        task: "db-backup".to_string(),
        metadata: Some(Metadata {
            db_name: "prod-db".to_string(),
            backup_location: "/path/to/backup".to_string(),
        }),
    };

    let any = Any {
        type_url: "type.googleapis.com/io.dapr.RustTest".to_string(),
        value: serde_json::to_vec(&job).unwrap(),
    };

    let job = JobBuilder::new("prod-db-backup")
        .with_schedule("@every 1s")
        .with_data(any)
        .build();

    let _schedule_resp = client.schedule_job_alpha1(job).await?;

    println!("job scheduled successfully");

    sleep(Duration::from_secs(3)).await;

    let get_resp = client.get_job_alpha1("prod-db-backup").await?;

    let get_resp_backup: Backup =
        serde_json::from_slice(&get_resp.clone().job.unwrap().data.unwrap().value).unwrap();

    println!("job retrieved: {:?}", get_resp_backup);

    let _delete_resp = client.delete_job_alpha1("prod-db-backup").await?;

    println!("job deleted");

    sleep(Duration::from_secs(5)).await;

    Ok(())
}
