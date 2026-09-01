use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;

const MAX_CONNECTIONS: usize = 100;
const TIMEOUT_DURATION: Duration = Duration::from_secs(10);
const MAX_LINE_SIZE: usize = 8192;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    tracing::info!("Server berjalan di http://127.0.0.1:8080");

    let counter = Arc::new(Mutex::new(0));
    let config = tokio_http_server::ServerConfig {
        max_connections: MAX_CONNECTIONS,
        timeout_duration: TIMEOUT_DURATION,
        max_line_size: MAX_LINE_SIZE,
    };
    tokio_http_server::run(listener, counter, config).await?;

    Ok(())
}
