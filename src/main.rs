use tokio::net::TcpListener;
use std::error::Error;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;   
    println!("Server berjalan di http://127.0.0.1:8080");

    let counter = Arc::new(Mutex::new(0));
    tokio_http_server::run(listener, counter).await?;

    Ok(())
}
