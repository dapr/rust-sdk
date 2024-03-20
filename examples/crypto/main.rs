use std::fs;

use tokio::time::sleep;

use dapr::dapr::dapr::proto::runtime::v1::{DecryptRequestOptions, EncryptRequestOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    sleep(std::time::Duration::new(2, 0)).await;
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

    let encrypted = client
        .encrypt(
            &"Test".to_string(),
            EncryptRequestOptions {
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

    let decrypted = client
        .decrypt(
            encrypted,
            DecryptRequestOptions {
                component_name: "localstorage".to_string(),
                key_name: "rsa-private-key.pem".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(String::from_utf8(decrypted).unwrap().as_str(), "Test");

    let image = fs::read("./examples/crypto/image.png").unwrap();

    let encrypted = client
        .encrypt(
            &image,
            EncryptRequestOptions {
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

    let decrypted = client
        .decrypt(
            encrypted,
            DecryptRequestOptions {
                component_name: "localstorage".to_string(),
                key_name: "rsa-private-key.pem".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(decrypted, image);

    Ok(())
}
