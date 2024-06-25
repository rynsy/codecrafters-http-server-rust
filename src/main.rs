use std::borrow::BorrowMut;
use std::env;
use std::error;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;

use http_server_starter_rust::parser::parse_http_request;
use http_server_starter_rust::types::*;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn handle_request(request: Request) -> Response {
    println!("Handling request: {:?}", request);
    let mut parts = request.path.split('/');
    let parts = parts.borrow_mut();
    let _ = parts.next(); // route starts with /
    let route = parts.next();
    match route {
        None => {
            println!("Couldn't match route: split failed");
            Response::new(StatusCode::Ok, "text/plain", "")
        }
        Some(s) => match s {
            "echo" => {
                let content = parts.next();
                let content = content.unwrap_or("");
                println!("[Echo] echoing: {}", content);
                Response::new(StatusCode::Ok, "text/plain", content)
            }
            "user-agent" => {
                println!(
                    "[user-agent] returning: {}",
                    request.headers.get("User-Agent").unwrap_or(&"".to_string())
                );
                let user_agent: &String = request.headers.get("User-Agent").unwrap();
                Response::new(StatusCode::Ok, "text/plain", user_agent)
            }
            "files" => {
                let filename = parts.next();
                let filename = filename.unwrap_or("");
                let directory = env::var("FILE_DIRECTORY").unwrap_or("".to_string());
                println!("[File] looking for: {} in {}", filename, directory);
                let path = Path::new(&directory);
                let path = path.join(filename);
                let file = File::open(path);
                let content = match file {
                    Ok(file) => {
                        let mut buf_reader = BufReader::new(file);
                        let mut contents = String::new();
                        buf_reader.read_to_string(&mut contents).unwrap();
                        contents
                    }
                    Err(e) => {
                        eprintln!("[File] Couldn't open file : {:?}", e);
                        "".to_string()
                    }
                };
                Response::new(StatusCode::Ok, "application/octet-stream", content.as_str())
            }
            "" => {
                println!("[/] default route. returning 200");
                Response::new(StatusCode::Ok, "text/plain", "")
            }
            _ => {
                println!("[ERROR] unknown route: {}", s);
                Response::new(StatusCode::NotFound, "text/plain", "")
            }
        },
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
    let separator = "\r\n";
    let status_line: &str = match response.status {
        StatusCode::Ok => "HTTP/1.1 200 OK",
        StatusCode::Forbidden => "HTTP/1.1 500 Forbidden",
        StatusCode::NotFound => "HTTP/1.1 404 Not Found",
        StatusCode::InternalServerError => "HTTP/1.1 500 Internal Server Error",
    };

    let mut response_str = "".to_string();
    response_str.push_str(status_line);
    response_str.push_str(separator);

    let mut content_type = "Content-Type: ".to_string();
    content_type.push_str(&response.content_type);
    response_str.push_str(&content_type);
    response_str.push_str(separator);

    let mut content_length = "Content-Length: ".to_string();
    content_length.push_str(&response.content_length);
    response_str.push_str(&content_length);
    response_str.push_str(separator);
    response_str.push_str(separator);

    response_str.push_str(&response.response_body);
    response_str.push_str(separator);
    response_str.push_str(separator);

    buf[..response_str.len()].copy_from_slice(response_str.as_bytes());
    response_str.len()
}

#[allow(clippy::never_loop)]
#[allow(clippy::redundant_guards)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    println!("Listening on port 4221....");

    let args: Vec<String> = env::args().collect();
    if args.len() > 2 && args[1] == "--directory" {
        println!("Setting folder environment variable: {}", args[2]);
        env::set_var("FILE_DIRECTORY", args[2].clone());
    }

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
