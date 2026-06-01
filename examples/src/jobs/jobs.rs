use std::time::Duration;

use dapr::appcallback::AppCallbackService;
use dapr::client::JobBuilder;
use dapr::dapr::proto::runtime::v1::{
    JobEventRequest, JobEventResponse, app_callback_alpha_server::AppCallbackAlphaServer,
    app_callback_server::AppCallbackServer,
};
use dapr::{add_job_handler, serde_json};
use prost_types::Any;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::sleep;
use tonic::Status;
use tonic::transport::Server;

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

async fn ping_pong_handler(_request: JobEventRequest) -> Result<JobEventResponse, Status> {
    println!("received job on ping_pong_handler");

    Ok(JobEventResponse::default())
}
async fn backup_job_handler(request: JobEventRequest) -> Result<JobEventResponse, Status> {
    // The logic for handling the backup job request

    if let Some(data) = request.data {
        // Deserialize the decoded value into a Backup struct
        let backup_val: Backup = serde_json::from_slice(&data.value).unwrap();

        println!("job received: {backup_val:?}");
    }

    Ok(JobEventResponse::default())
}

#[tokio::main]
#[allow(non_camel_case_types)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(async move {
        let server_addr = "127.0.0.1:50051".parse().unwrap();

        println!("AppCallback server listening on {server_addr}");

        let mut callback_service = AppCallbackService::new();

        let backup_job_handler_name = "prod-db-backup";
        add_job_handler!(
            callback_service,
            backup_job_handler_name,
            backup_job_handler
        );

        let ping_pong_handler_name = "ping-pong";
        add_job_handler!(callback_service, ping_pong_handler_name, ping_pong_handler);

        let callback_service = Arc::new(callback_service);

        Server::builder()
            .add_service(AppCallbackServer::from_arc(callback_service.clone()))
            .add_service(AppCallbackAlphaServer::from_arc(callback_service))
            .serve(server_addr)
            .await
            .unwrap();
    });

    sleep(Duration::from_secs(5)).await;

    // Client

    let address = dapr::client::default_sidecar_address();
    println!("attempting to create a dapr client: {address}");

    // Create the client
    let mut client = dapr::Client::new().await?;

    println!("client created");

    // define job data in json
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

    let _schedule_resp = client.schedule_job(job, None).await?;

    println!("job scheduled successfully");

    sleep(Duration::from_secs(3)).await;

    let get_resp = client.get_job("prod-db-backup").await?;

    let get_resp_backup: Backup =
        serde_json::from_slice(&get_resp.clone().job.unwrap().data.unwrap().value).unwrap();

    println!("job retrieved: {get_resp_backup:?}");

    let _delete_resp = client.delete_job("prod-db-backup").await?;

    println!("job deleted");

    sleep(Duration::from_secs(5)).await;

    // Second handler

    let ping_pong_job = JobBuilder::new("ping-pong")
        .with_schedule("@every 1s")
        .with_repeats(5)
        .build();
    let _schedule_resp = client.schedule_job(ping_pong_job, None).await?;

    sleep(Duration::from_secs(10)).await;

    Ok(())
}
