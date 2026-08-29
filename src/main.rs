use tokio::net::TcpListener;
use std::error::Error;
use std::time::Duration;
use std::sync::{Arc, Mutex};

const MAX_CONNECTIONS: usize = 100;
const TIMEOUT_DURATION: Duration = Duration::from_secs(10);
const MAX_LINE_SIZE: usize = 8192;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;   
    println!("Server berjalan di http://127.0.0.1:8080");

    let counter = Arc::new(Mutex::new(0));
    let config = tokio_http_server::ServerConfig {
        max_connections: MAX_CONNECTIONS,
        timeout_duration: TIMEOUT_DURATION,
        max_line_size: MAX_LINE_SIZE
    };
    tokio_http_server::run(listener, counter, config).await?;

    Ok(())
}
