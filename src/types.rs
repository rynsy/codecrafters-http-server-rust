use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug)]
pub struct Request {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}
#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Decompression error: {0}")]
    DecompressionError(String),
    #[error("Handling error: {0}")]
    HandlingError(String),
    #[error("Compression error: {0}")]
    CompressionError(String),
}

#[derive(Debug, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PATCH,
    PUT,
    DELETE,
    UNKNOWN,
}
#[derive(Debug, PartialEq)]
pub enum EncodingScheme {
    GZIP,
    NONE,
}
#[derive(Debug)]
pub enum ResponseStatus {
    Ok = 200,
    Created = 201,
    BadRequest = 400,
    NotFound = 404,
    InternalServerError = 500,
}
#[derive(Debug)]
pub struct Response {
    pub status: ResponseStatus,
    pub content_type: Box<str>,
    pub content_length: Box<str>,
    pub content_encoding: Box<str>,
    pub response_body: Box<str>,
}

impl Response {
    pub fn new(
        status: ResponseStatus,
        content_type: &str,
        content_encoding: &str,
        response_body: &str,
    ) -> Self {
        let content_len = response_body.len();
        let content_len = content_len.to_string();
        let content_len: &str = &content_len;

        Response {
            status,
            content_type: Box::from(content_type),
            content_length: Box::from(content_len),
            content_encoding: Box::from(content_encoding),
            response_body: Box::from(response_body),
        }
    }
}
