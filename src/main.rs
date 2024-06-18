use std::error;

use http_server_starter_rust::parser::parse_http_request;
use http_server_starter_rust::types::*;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn handle_request(request: Request) -> Response {
    if request.path == "/" {
        Response {
            status: StatusCode::Ok,
        }
    } else {
        Response {
            status: StatusCode::NotFound,
        }
    }
}

fn handle_client(buf: &[u8], n: usize) -> Option<Response> {
    if n == 0 {
        eprintln!("Client handler got 0 bytes from client");
        return None;
    }
    let data: Vec<u8> = buf[..n].to_vec();
    let request = String::from_utf8(data).unwrap_or("".to_string());
    match parse_http_request(request.as_str()) {
        Ok((_, req)) => Some(handle_request(req)),
        Err(_) => {
            eprintln!("parse_http_request failed to parse the request");
            None
        }
    }
}

fn response_to_bytes(buf: &mut [u8], response: Response) -> usize {
    let bytes = match response.status {
        StatusCode::Ok => "HTTP/1.1 200 OK\r\n\r\n",
        StatusCode::Forbidden => "HTTP/1.1 500 Forbidden\r\n\r\n",
        StatusCode::NotFound => "HTTP/1.1 404 Not Found\r\n\r\n",
        StatusCode::InternalServerError => "HTTP/1.1 500 Internal Server Error\r\n\r\n",
    }
    .as_bytes();

    buf[..bytes.len()].copy_from_slice(bytes);
    bytes.len()
}

#[allow(clippy::never_loop)]
#[allow(clippy::redundant_guards)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    println!("Listening on port 4221....");

    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => {
                        println!("Connection closed by peer");
                        return;
                    }
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
                match handle_client(&buf, n) {
                    Some(response) => {
                        let n = response_to_bytes(&mut buf, response);
                        if let Err(e) = socket.write(&buf[..n]).await {
                            eprintln!("failed to write to socket; err = {:?}", e);
                            return;
                        }
                        println!("Response sent. Closing connection");
                        break;
                    }
                    None => {
                        eprintln!("Client handler gave empty response");
                        return;
                    }
                }
            }
        });
    }
}
