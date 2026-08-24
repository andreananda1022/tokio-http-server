use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use std::error::Error;
use std::time::Duration;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String
}

#[derive(Debug)]
pub enum ParseError {
    InvalidRequest(String)
}

pub fn parse_request_line(line: &str) -> Result<Request, ParseError> {
    let trimmed_line = line.trim_end();
    let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(ParseError::InvalidRequest(format!("Invalid request line: {trimmed_line}")));
    }
    
    Ok(Request {
        method: parts[0].to_string(),
        path: parts[1].to_string(),
        version: parts[2].to_string()
    })
}

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

    let request = match parse_request_line(&request_line) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Invalid request: {e:#?}");
            let body = "Request tidak valid!";
            let response = format!("HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\n\r\n{}", body.len(), body);
            socket.write_all(response.as_bytes()).await?;
            return Ok(());
        }
    };

    let response = if request.method == "GET" && request.path == "/slow" {
        println!("--> Memulai proses lambat (5 detik)...");
        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("--> Proses lambat selesai.");
        let body = "Proses lambat";
        format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", body.len(), body)
    } else if request.method == "GET" && request.path == "/" {
        let current_visitor = {
            let mut visitor = counter.lock().unwrap();
            *visitor += 1;
            *visitor
        };
        let body = format!("Beranda\nVisitor: {}", current_visitor);
        format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", body.len(), body)
    } else {
        let body = "Halaman tidak ditemukan!";
        format!("HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\n\r\n{}", body.len(), body)
    };

    socket.write_all(response.as_bytes()).await?;
    Ok(())
}

pub async fn run(listener: TcpListener, counter: Arc<Mutex<usize>>) -> Result<(), Box<dyn Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_get_request() {
        let input = "GET / HTTP/1.1";
        let result = parse_request_line(input).unwrap();
        assert_eq!(result.method, "GET");
        assert_eq!(result.path, "/");
        assert_eq!(result.version, "HTTP/1.1");
    }

    #[test]
    fn rejects_request_line_with_too_few_parts() {
        let input = "GET /";
        let result = parse_request_line(input);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_request_line_with_too_many_parts() {
        let input = "GET / HTTP/1.1 EXTRA";
        let result = parse_request_line(input);
        assert!(result.is_err());
    }

    #[test]
    fn parses_request_with_double_whitespace() {
        let input = "GET  / HTTP/1.1  ";
        let result = parse_request_line(input).unwrap();
        assert_eq!(result.method, "GET");
        assert_eq!(result.path, "/");
        assert_eq!(result.version, "HTTP/1.1");
    }
}
