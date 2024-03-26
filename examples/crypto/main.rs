use std::fs;

use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::time::sleep;

use dapr::client::ReaderStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    sleep(std::time::Duration::new(2, 0)).await;
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

    let encrypted = client
        .encrypt(
            ReaderStream::new("Test".as_bytes()),
            dapr::client::EncryptRequestOptions {
                component_name: "localstorage".to_string(),
                key_name: "rsa-private-key.pem".to_string(),
                key_wrap_algorithm: "RSA".to_string(),
                data_encryption_cipher: "aes-gcm".to_string(),
                omit_decryption_key_name: false,
                decryption_key_name: "rsa-private-key.pem".to_string(),
            },
        )
        .await
        .unwrap();

    let mut decrypted = client
        .decrypt(
            encrypted,
            dapr::client::DecryptRequestOptions {
                component_name: "localstorage".to_string(),
                key_name: "rsa-private-key.pem".to_string(),
            },
        )
        .await
        .unwrap();

    let mut value = String::new();

    decrypted.read_to_string(&mut value).await.unwrap();

    assert_eq!(value.as_str(), "Test");

    println!("Successfully Decrypted String");

    let image = File::open("./image.png").await.unwrap();

    let encrypted = client
        .encrypt(
            ReaderStream::new(image),
            dapr::client::EncryptRequestOptions {
                component_name: "localstorage".to_string(),
                key_name: "rsa-private-key.pem".to_string(),
                key_wrap_algorithm: "RSA".to_string(),
                data_encryption_cipher: "aes-gcm".to_string(),
                omit_decryption_key_name: false,
                decryption_key_name: "rsa-private-key.pem".to_string(),
            },
        )
        .await
        .unwrap();

    let mut decrypted = client
        .decrypt(
            encrypted,
            dapr::client::DecryptRequestOptions {
                component_name: "localstorage".to_string(),
                key_name: "rsa-private-key.pem".to_string(),
            },
        )
        .await
        .unwrap();

    let image = fs::read("./image.png").unwrap();

    let mut buf = bytes::BytesMut::with_capacity(image.len());

    decrypted.read_buf(&mut buf).await.unwrap();

    assert_eq!(buf.to_vec(), image);

    println!("Successfully Decrypted Image");

    Ok(())
}
