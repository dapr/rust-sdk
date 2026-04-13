use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::sleep(Duration::from_secs(5)).await;

    let address = "https://127.0.0.1".to_string();

    let client_result = dapr::Client::<dapr::client::TonicClient>::connect(address).await;
    
    let value = "test";
    let method_to_call = format!("hello/{}", value);

    match client_result {
        Ok(mut _client) => {
            let http_client = reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap();

            let target_app_id = "invoke-http-server";
            let dapr_http_port: u16 = std::env::var("DAPR_HTTP_PORT")
                .unwrap_or_else(|_| "3500".to_string())
                .parse()?;

            let url = format!("http://127.0.0.1:{}/{}", dapr_http_port, method_to_call);
            let req = http_client.post(url).header("dapr-app-id", target_app_id);

            let response_result = req.send().await;
            match response_result {
                Ok(response) => {
                    let body_result = response.text().await;
                    match body_result {
                        Ok(body) => {
                            println!("Response: {:?}", body);
                        }
                        Err(e) => {
                            eprintln!("Error 1: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error 2: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Dapr client error: {}", e);
        }
    }

    Ok(())
}
