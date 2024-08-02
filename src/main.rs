use flate2::write::GzEncoder;
use flate2::Compression;
use nom::AsBytes;
use std::borrow::BorrowMut;
use std::env;
use std::error;
use std::io::Error;
use std::io::Write;
use std::path::PathBuf;
use tokio::fs;

use http_server_starter_rust::parser::parse_http_request;
use http_server_starter_rust::types::*;

use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn handle_request(request: Request) -> Result<Response, RequestError> {
    let compression_scheme = _handle_compression_header(&request)
        .await
        .unwrap_or(EncodingScheme::NONE);
    let decompressed_request = _decompress_request(request)
        .await
        .map_err(|e| RequestError::DecompressionError(e.to_string()))?;

    let response = _handle_request(decompressed_request)
        .await
        .map_err(|e| RequestError::HandlingError(e.to_string()))?;

    let compressed_response = _compress_response(compression_scheme, response)
        .await
        .map_err(|e| RequestError::CompressionError(e.to_string()))?;

    Ok(compressed_response)
}

async fn _handle_compression_header(
    request: &Request,
) -> Result<EncodingScheme, Box<dyn std::error::Error>> {
    let compression_scheme = match request.headers.get("Accept-Encoding") {
        Some(encodings) => {
            // TODO: Walk through CSV and find a valid scheme
            //
            let trimmed_encodings: Vec<&str> = encodings
                .split(',')
                .map(|e| e.trim())
                .filter(|e| e.to_lowercase() == "gzip")
                .collect();

            if !trimmed_encodings.is_empty() {
                EncodingScheme::GZIP
            } else {
                EncodingScheme::NONE
            }
        }
        None => EncodingScheme::NONE,
    };

    Ok(compression_scheme)
}

async fn _decompress_request(request: Request) -> Result<Request, Box<dyn std::error::Error>> {
    // Implement decompression logic here
    Ok(request)
}

async fn _compress_response(
    encoding: EncodingScheme,
    response: Response,
) -> Result<Response, Box<dyn std::error::Error>> {
    match encoding {
        EncodingScheme::GZIP => {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(response.response_body.as_slice())?;
            let compressed_body = encoder.finish()?;
            let compressed_response = Response::new(
                response.status,
                &response.content_type,
                "gzip",
                compressed_body,
            );

            Ok(compressed_response)
        }
        EncodingScheme::NONE => Ok(response),
    }
}

async fn _handle_request(request: Request) -> Result<Response, Error> {
    println!("Handling request: {:?}", request);
    let mut parts = request.path.split('/');
    let parts = parts.borrow_mut();
    let _ = parts.next(); // route starts with /
    let route = parts.next();
    match route {
        None => {
            println!("Couldn't match route: split failed");
            Ok(Response::new(
                ResponseStatus::Ok,
                "text/plain",
                "",
                Vec::new(),
            ))
        }
        Some(path) => match (request.method, path) {
            (HttpMethod::GET, "echo") => {
                let content = parts.next().unwrap_or("");
                println!("[Echo] echoing: {}", content);
                let content = content.as_bytes().to_vec();
                Ok(Response::new(ResponseStatus::Ok, "text/plain", "", content))
            }
            (HttpMethod::GET, "user-agent") => {
                println!(
                    "[user-agent] returning: {}",
                    request.headers.get("User-Agent").unwrap_or(&"".to_string())
                );
                let user_agent = request
                    .headers
                    .get("User-Agent")
                    .unwrap()
                    .as_bytes()
                    .to_vec();
                Ok(Response::new(
                    ResponseStatus::Ok,
                    "text/plain",
                    "",
                    user_agent,
                ))
            }
            (HttpMethod::GET, "files") => {
                let filename = parts.next();
                let filename = filename.unwrap_or("");

                if filename.is_empty() {
                    return Ok(Response::new(
                        ResponseStatus::NotFound,
                        "text/plain",
                        "",
                        Vec::new(),
                    ));
                }

                let directory = env::var("FILE_DIRECTORY").unwrap_or("".to_string());
                let mut readdir = fs::read_dir(directory).await?;

                while let Some(entry) = readdir.next_entry().await? {
                    if entry.file_name().to_str().unwrap() == filename {
                        let contents = fs::read_to_string(entry.path()).await?;
                        let contents = contents.as_bytes().to_vec();
                        return Ok(Response::new(
                            ResponseStatus::Ok,
                            "application/octet-stream",
                            "",
                            contents,
                        ));
                    }
                }
                Ok(Response::new(
                    ResponseStatus::NotFound,
                    "text/plain",
                    "",
                    Vec::new(),
                ))
            }
            (HttpMethod::POST, "files") => {
                let filename = parts.next();
                let filename = filename.unwrap_or("");

                if filename.is_empty() {
                    return Ok(Response::new(
                        ResponseStatus::BadRequest,
                        "text/plain",
                        "",
                        Vec::new(),
                    ));
                }

                let directory = env::var("FILE_DIRECTORY").unwrap_or("".to_string());
                let path = PathBuf::from(directory).join(filename);

                println!(
                    "[POST files] creating file {:?} with content: {:?}",
                    path, request.body
                );

                match File::create(path).await {
                    Ok(mut file) => {
                        let r = file.write_all(request.body.as_bytes()).await;
                        println!("[POST files] Successfully saved file, result {:?}", r,);
                    }
                    Err(s) => {
                        eprintln!("[POST files] Error saving file, error {:?}", s,);
                        return Ok(Response::new(
                            ResponseStatus::BadRequest,
                            "text/plain",
                            "",
                            Vec::new(),
                        ));
                    }
                }

                Ok(Response::new(
                    ResponseStatus::Created,
                    "text/plain",
                    "",
                    Vec::new(),
                ))
            }
            (HttpMethod::GET, "") => {
                println!("[/] default route. returning 200");
                Ok(Response::new(
                    ResponseStatus::Ok,
                    "text/plain",
                    "",
                    Vec::new(),
                ))
            }
            (HttpMethod::UNKNOWN, _) => {
                println!("[ERROR] unknown method");
                Ok(Response::new(
                    ResponseStatus::BadRequest,
                    "text/plain",
                    "",
                    Vec::new(),
                ))
            }
            (_, s) => {
                println!("[ERROR] unknown route: {}", s);
                Ok(Response::new(
                    ResponseStatus::NotFound,
                    "text/plain",
                    "",
                    Vec::new(),
                ))
            }
        },
    }
}

async fn handle_client(buf: &[u8], n: usize) -> Option<Response> {
    if n == 0 {
        eprintln!("Client handler got 0 bytes from client");
        return None;
    }
    let data: Vec<u8> = buf[..n].to_vec();
    let request = String::from_utf8(data).unwrap_or("".to_string());
    match parse_http_request(request.as_str()) {
        Ok((_, req)) => match handle_request(req).await {
            Ok(response) => Some(response),
            Err(e) => {
                eprintln!("handle_request returned an error: {}", e);
                None
            }
        },
        Err(_) => {
            eprintln!("parse_http_request failed to parse the request");
            None
        }
    }
}

fn response_to_bytes(buf: &mut [u8], response: Response) -> usize {
    let separator = "\r\n";
    let status_line: &str = match response.status {
        ResponseStatus::Ok => "HTTP/1.1 200 OK",
        ResponseStatus::Created => "HTTP/1.1 201 Created",
        ResponseStatus::BadRequest => "HTTP/1.1 500 Forbidden",
        ResponseStatus::NotFound => "HTTP/1.1 404 Not Found",
        ResponseStatus::InternalServerError => "HTTP/1.1 500 Internal Server Error",
    };

    let mut response_str = "".to_string();
    response_str.push_str(status_line);
    response_str.push_str(separator);

    let mut content_type = "Content-Type: ".to_string();
    content_type.push_str(&response.content_type);
    response_str.push_str(&content_type);
    response_str.push_str(separator);

    if !response.content_encoding.is_empty() {
        let mut content_encoding = "Content-Encoding: ".to_string();
        content_encoding.push_str(&response.content_encoding);
        response_str.push_str(&content_encoding);
        response_str.push_str(separator);
    }

    let mut content_length = "Content-Length: ".to_string();
    content_length.push_str(&response.content_length);
    response_str.push_str(&content_length);
    response_str.push_str(separator);
    response_str.push_str(separator);

    let response_bytes = [response_str.as_bytes(), &response.response_body].concat();

    buf[..response_bytes.len()].copy_from_slice(&response_bytes);
    response_bytes.len()
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
                match handle_client(&buf, n).await {
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
