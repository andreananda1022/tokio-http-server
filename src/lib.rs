use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::sync::Semaphore;
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::time::Duration;

#[derive(Debug)]
pub enum ParseError {
    InvalidRequest(String)
}

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String
}

#[derive(Debug, Clone, Copy)]
pub struct ServerConfig {
    pub max_connections: usize,
    pub timeout_duration: Duration,
    pub max_line_size: usize,
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

async fn read_request_line(socket: &mut TcpStream, config: ServerConfig) -> std::io::Result<Option<String>> {
    let mut request_line = String::new();
    let buf_reader = BufReader::new(socket);
    let mut limited_reader = buf_reader.take(config.max_line_size as u64);

    let read_result = tokio::time::timeout(config.timeout_duration, limited_reader.read_line(&mut request_line)).await;

    match read_result {
        Ok(Ok(0)) => {
            Ok(None)
        }
        Ok(Ok(_n)) => {
            if request_line.ends_with("\n") {
                Ok(Some(request_line))
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Error: Request length exceeds the {} bytes limit.", config.max_line_size)))
            }
        }
        Ok(Err(e)) => {
            Err(e)
        }
        Err(_) => {
            Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Request time out!"))
        }

    }
}

async fn handle_connection(mut socket: TcpStream, counter: Arc<Mutex<usize>>, config: ServerConfig) -> Result<(), Box<dyn Error>> {
    let request_line = match read_request_line(&mut socket, config).await? {
        Some(line) => line,
        None => return Ok(())
    };
    
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

pub async fn run(listener: TcpListener, counter: Arc<Mutex<usize>>, config: ServerConfig) -> Result<(), Box<dyn Error>> {
    let semaphore = Arc::new(Semaphore::new(config.max_connections));
    
    loop {
        let (mut socket, addr) = listener.accept().await?;
        let counter = Arc::clone(&counter);
        let semaphore = Arc::clone(&semaphore);

        match semaphore.try_acquire_owned() {
            Ok(permit) => {
                tokio::spawn(async move {
                    let _permit = permit;
                    if let Err(e) = handle_connection(socket, counter, config).await {
                        eprintln!("Error pada klien {}: {}", addr, e);
                    }
                });
            }
            Err(_) => {
                tokio::spawn(async move {
                    match read_request_line(&mut socket, config).await {
                        Ok(Some(_)) => {
                            let body = "Server Busy: Overload";
                            let response = format!("HTTP/1.1 503 Service Unavailable\r\nContent-Length: {}\r\n\r\n{}", body.len(), body);
                            if let Err(e) = socket.write_all(response.as_bytes()).await {
                                eprintln!("Gagal mengirim 503 ke klien {}: {}", addr, e);
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            eprintln!("Gagal membaca karena kesalahan I/O: {e}");
                        }
                    }
                });
            }
        }
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
