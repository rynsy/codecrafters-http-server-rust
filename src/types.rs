use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
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
    pub response_body: Box<str>,
}

impl Response {
    pub fn new(status: ResponseStatus, content_type: &str, response_body: &str) -> Self {
        let content_len = response_body.len();
        let content_len = content_len.to_string();
        let content_len: &str = &content_len;

        Response {
            status,
            content_type: Box::from(content_type),
            content_length: Box::from(content_len),
            response_body: Box::from(response_body),
        }
    }
}
