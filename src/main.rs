use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use std::error::Error;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use tokio_http_server::{Request, parse_request_line};

async fn handle_connection(mut socket: TcpStream, counter: Arc<Mutex<usize>>) -> Result<(), Box<dyn Error>> {
    let mut request_line = String::new();
    
    {
        let mut buf_reader = BufReader::new(&mut socket);
        let n = buf_reader.read_line(&mut request_line).await?;
        if n == 0 {
            return Ok(());
        }
    }

    println!("Request: {}", request_line.trim_end());

    let response = if request_line.starts_with("GET /slow HTTP/1.1") {
        println!("--> Memulai proses lambat (5 detik)...");
        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("--> Proses lambat selesai.");
        String::from("HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nProses Lambat")
        
    } else if request_line.starts_with("GET / HTTP/1.1") {
        let current_visitor = {
            let mut visitor = counter.lock().unwrap();
            *visitor += 1;
            *visitor 
        };
        
        let body = format!("Beranda\nVisitor: {}", current_visitor);
        
        format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", body.len(), body)
        
    } else {
        String::from("HTTP/1.1 404 NOT FOUND\r\nContent-Length: 15\r\n\r\nTidak Ditemukan")
    };

    socket.write_all(response.as_bytes()).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server berjalan di http://127.0.0.1:8080");

    let counter = Arc::new(Mutex::new(0));

    loop {
        let (socket, addr) = listener.accept().await?;
        let counter = Arc::clone(&counter);
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, counter).await {
                eprintln!("Error pada klien {}: {}", addr, e);
            }
        });
    }
}
