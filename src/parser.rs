use crate::{types::Request, HttpMethod};
use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_while},
    character::complete::{
        alphanumeric1, line_ending, multispace0, not_line_ending, space0, space1,
    },
    combinator::rest,
    multi::many_till,
    sequence::{pair, tuple},
    IResult,
};

fn is_token_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '.'
}

fn parse_method(input: &str) -> IResult<&str, &str> {
    alphanumeric1(input)
}

fn parse_path(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| is_token_char(c) || c == '_' || c == '/' || c == '?')(input)
}

fn parse_version(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("HTTP/")(input)?;
    take_while(|c: char| c.is_ascii_digit() || c == '.')(input)
}

fn parse_header_name(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| is_token_char(c))(input)
}

fn parse_header(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, (name, _, _, value, _)) = tuple((
        parse_header_name,
        tag(":"),
        space0,
        not_line_ending,
        pair(line_ending, space0),
    ))(input)?;
    Ok((input, (name, value)))
}

pub fn parse_http_request(input: &str) -> IResult<&str, Request> {
    let (input, (method, _, path, _, version, _, (headers, _), body)) = tuple((
        parse_method,
        space1,
        parse_path,
        space1,
        parse_version,
        multispace0,
        many_till(parse_header, pair(line_ending, multispace0)),
        rest,
    ))(input)?;

    let method = match method.to_uppercase().as_str() {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        "PATCH" => HttpMethod::PATCH,
        "PUT" => HttpMethod::PUT,
        "DELETE" => HttpMethod::DELETE,
        _ => HttpMethod::UNKNOWN,
    };

    let headers = headers
        .into_iter()
        .map(|(name, value)| (name.to_string(), value.trim().to_string()))
        .collect::<HashMap<_, _>>();

    Ok((
        input,
        Request {
            method,
            path: path.replace('\\', "/").to_string(),
            version: version.to_string(),
            headers,
            body: body.trim().to_string(),
        },
    ))
}

#[test]
fn parse_http() {
    let input = "GET /tweets HTTP/1.1
            User-Agent: PostmanRuntime/7.32.3
            Accept: */*
            Cache-Control: no-cache
            Postman-Token: 953cc42d-60e6-4155-ab58-54d958c62304
            Host: localhost:4221
            Accept-Encoding: gzip, deflate, br
            Connection: keep-alive
            \r\n
        
            Test Body
        ";

    match parse_http_request(input) {
        Ok((_, result)) => {
            assert_eq!(result.method, HttpMethod::GET);
            assert_eq!(result.path, "/tweets");
            assert_eq!(result.version, "1.1");
            assert_eq!(
                result.headers.get("User-Agent").unwrap(),
                "PostmanRuntime/7.32.3"
            );
            assert_eq!(result.headers.get("Accept").unwrap(), "*/*");
            assert_eq!(result.headers.get("Cache-Control").unwrap(), "no-cache");
            assert_eq!(
                result.headers.get("Postman-Token").unwrap(),
                "953cc42d-60e6-4155-ab58-54d958c62304"
            );
            assert_eq!(result.headers.get("Host").unwrap(), "localhost:4221");
            assert_eq!(
                result.headers.get("Accept-Encoding").unwrap(),
                "gzip, deflate, br"
            );
            assert_eq!(result.headers.get("Connection").unwrap(), "keep-alive");
            assert_eq!(result.body, "Test Body");
        }
        Err(e) => {
            eprintln!("Error! {:?}", e);
            assert_eq!(true, false);
        }
    }
}
#[test]
fn parse_post_request_body() {
    let input = "POST /files/file_123 HTTP/1.1                                                                 Host: localhost:2020
User-Agent: curl/8.8.0
Accept: */*
Content-Type: application/octet-stream
Content-Length: 5

12345";
    match parse_http_request(input) {
        Ok((_, result)) => {
            assert_eq!(result.method, HttpMethod::POST);
            assert_eq!(result.path, "/files/file_123");
            assert_eq!(result.version, "1.1");
            assert_eq!(result.headers.get("User-Agent").unwrap(), "curl/8.8.0");
            assert_eq!(result.headers.get("Accept").unwrap(), "*/*");
            assert_eq!(
                result.headers.get("Content-Type").unwrap(),
                "application/octet-stream"
            );
            assert_eq!(result.headers.get("Content-Length").unwrap(), "5");
            assert_eq!(result.body, "12345");
        }
        Err(e) => {
            eprintln!("Error! {:?}", e);
            assert_eq!(true, false);
        }
    }
}
