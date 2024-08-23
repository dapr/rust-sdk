use dapr::client_new::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::new().await.unwrap();
    println!("Client initialised");

    let metadata = client.get_metadata().await;

    println!("{:?}", metadata.unwrap())
}
