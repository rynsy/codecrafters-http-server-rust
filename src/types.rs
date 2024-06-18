#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
}
pub enum StatusCode {
    Ok = 200,
    Forbidden = 400,
    NotFound = 404,
    InternalServerError = 500,
}
pub struct Response {
    pub status: StatusCode,
}
