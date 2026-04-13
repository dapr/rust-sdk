use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await?;
    println!("Server running on {}", addr);

    loop {
        let (mut stream, _) = listener.accept().await?;
        
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            if let Ok(n) = stream.read(&mut buf).await {
                if n > 0 {
                    let request_str = String::from_utf8_lossy(&buf[..n]);
                    
                    if let Some(line) = request_str.lines().next() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let path = parts[1];
                            
                            if let Some(value) = path.strip_prefix("/hello/") {
                                let body = format!("Hello, {}! (from HTTP server)", value);
                                let response = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                    body.len(),
                                    body
                                );
                                let _ = stream.write_all(response.as_bytes()).await;
                                return;
                            }
                        }
                    }
                    
                    let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            }
        });
    }
}
