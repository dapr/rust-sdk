use std::fs;

use tokio::fs::File;

use dapr::client::ReaderStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "https://127.0.0.1".to_string();

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

    let decrypted = client
        .decrypt(
            encrypted,
            dapr::client::DecryptRequestOptions {
                component_name: "localstorage".to_string(),
                key_name: "rsa-private-key.pem".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(String::from_utf8(decrypted).unwrap().as_str(), "Test");

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

    let decrypted = client
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

    assert_eq!(decrypted, image);

    println!("Successfully Decrypted Image");

    Ok(())
}
