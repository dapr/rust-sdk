use std::time::Duration;

use dapr::client::{JobBuilder, JobFailurePolicyBuilder, JobFailurePolicyType};
use dapr::dapr::proto::runtime::v1::{
    app_callback_alpha_server::AppCallbackAlphaServer, JobEventRequest, JobEventResponse,
};
use dapr::server::appcallbackalpha::{AppCallbackServiceAlpha, JobHandlerMethod};
use dapr::{add_job_handler_alpha, serde_json};
use prost_types::Any;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tonic::transport::Server;
use tonic::Status;

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

async fn ping_pong_handler(_request: JobEventRequest) -> Result<JobEventResponse, Status> {
    println!("received job on ping_pong_handler");

    Ok(JobEventResponse::default())
}
async fn backup_job_handler(request: JobEventRequest) -> Result<JobEventResponse, Status> {
    // The logic for handling the backup job request

    if request.data.is_some() {
        // Deserialize the decoded value into a Backup struct
        let backup_val: Backup = serde_json::from_slice(&request.data.unwrap().value).unwrap();

        println!("job received: {backup_val:?}");
    }

    // Return a failure response to simulate a job failure
    Err(Status::internal("Simulated job failure"))
}

#[tokio::main]
#[allow(non_camel_case_types)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(async move {
        let server_addr = "127.0.0.1:50051".parse().unwrap();

        println!("AppCallbackAlpha server listening on {server_addr}");

        let mut alpha_callback_service = AppCallbackServiceAlpha::new();

        let backup_job_handler_name = "prod-db-backup";
        add_job_handler_alpha!(
            alpha_callback_service,
            backup_job_handler_name,
            backup_job_handler
        );

        let ping_pong_handler_name = "ping-pong";
        add_job_handler_alpha!(
            alpha_callback_service,
            ping_pong_handler_name,
            ping_pong_handler
        );

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
    let address = format!("{client_addr}:{port}");

    println!("attempting to create a dapr client: {address}");

    // Create the client
    let mut client = DaprClient::connect(client_addr).await?;

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
        .with_failure_policy(JobFailurePolicyBuilder::new(JobFailurePolicyType::Drop {}).build())
        .build();

    let _schedule_resp = client.schedule_job_alpha1(job, None).await?;

    println!("job scheduled successfully");

    sleep(Duration::from_secs(3)).await;

    let get_resp = client.get_job_alpha1("prod-db-backup").await?;

    let get_resp_backup: Backup =
        serde_json::from_slice(&get_resp.clone().job.unwrap().data.unwrap().value).unwrap();

    println!("job retrieved: {get_resp_backup:?}");

    let _delete_resp = client.delete_job_alpha1("prod-db-backup").await?;

    println!("job deleted");

    sleep(Duration::from_secs(5)).await;

    // Second handler

    let ping_pong_job = JobBuilder::new("ping-pong")
        .with_schedule("@every 1s")
        .with_repeats(5)
        .build();
    let _schedule_resp = client.schedule_job_alpha1(ping_pong_job, None).await?;

    sleep(Duration::from_secs(10)).await;

    Ok(())
}
