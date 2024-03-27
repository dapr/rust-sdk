use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    sleep(std::time::Duration::new(2, 0)).await;
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

    let files = vec![("my-data", b"some-data".to_vec())];

    client.save_state("statestore", files).await.unwrap();

    let result = client
        .lock(dapr::client::TryLockRequest {
            store_name: "lockstore".to_string(),
            resource_id: "my-data".to_string(),
            lock_owner: "some-random-id".to_string(),
            expiry_in_seconds: 60,
        })
        .await
        .unwrap();

    assert!(result.success);

    println!("Successfully locked my-data");

    let result = client
        .unlock(dapr::client::UnlockRequest {
            store_name: "lockstore".to_string(),
            resource_id: "my-data".to_string(),
            lock_owner: "some-random-id".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(0, result.status);

    println!("Successfully unlocked my-data");

    Ok(())
}
