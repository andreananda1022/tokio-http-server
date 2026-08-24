use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::{Arc, Mutex};

async fn spawn_test_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let counter = Arc::new(Mutex::new(0));

    tokio::spawn(async move {
        if let Err(e) = tokio_http_server::run(listener, counter).await {
            eprintln!("Internal server error\n{e}");
        }
    });

    port
}

#[tokio::test]
async fn get_root_returns_200_with_visitor_count() {
    let port = spawn_test_server().await;
    let server_addr = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect(server_addr).await.unwrap();

    let request_line = "GET / HTTP/1.1\r\n\r\n";
    stream.write_all(request_line.as_bytes()).await.unwrap();

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    assert!(response.contains("200 OK"));
    assert!(response.contains("Visitor: 1"));
}

#[tokio::test]
async fn rejects_malformed_request_with_400() {
    let port = spawn_test_server().await;
    let server_addr = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect(server_addr).await.unwrap();

    let request_line = "GET / HTTP/1.1 EXTRA\r\n\r\n";
    stream.write_all(request_line.as_bytes()).await.unwrap();

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    assert!(response.contains("400 Bad Request"));
}

#[tokio::test]
#[ignore]
async fn slow_endpoint_delays_response_by_5_seconds() {
    let start = std::time::Instant::now();

    let port = spawn_test_server().await;
    let server_addr = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect(server_addr).await.unwrap();

    let request_line = "GET /slow HTTP/1.1\r\n\r\n";
    stream.write_all(request_line.as_bytes()).await.unwrap();

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    let elapsed = start.elapsed();

    assert!(response.contains("200 OK"));
    assert!(elapsed.as_secs() >= 5);
}

#[tokio::test]
async fn returns_404_for_unknown_route() {
    let port = spawn_test_server().await;
    let server_addr = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect(server_addr).await.unwrap();

    let request_line = "GET /something HTTP/1.1\r\n\r\n";
    stream.write_all(request_line.as_bytes()).await.unwrap();

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    assert!(response.contains("404 Not Found"));
    assert!(response.contains("Halaman tidak ditemukan!"));
}

#[tokio::test]
async fn server_closes_connection_when_no_data_sent() {
    let port = spawn_test_server().await;
    let server_addr = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect(server_addr).await.unwrap();

    stream.shutdown().await.unwrap();

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await.unwrap();

    assert_eq!(n, 0);
}
