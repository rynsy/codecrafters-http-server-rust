#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub user_agent: String,
}
#[derive(Debug)]
pub enum StatusCode {
    Ok = 200,
    Forbidden = 400,
    NotFound = 404,
    InternalServerError = 500,
}
#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub content_type: Box<str>,
    pub content_length: Box<str>,
    pub response_body: Box<str>,
}

impl Response {
    pub fn new(status: StatusCode, content_type: &str, response_body: &str) -> Self {
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
