use crate::{types::Request, HttpMethod};
use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_while},
    character::complete::{
        alphanumeric1, line_ending, multispace0, not_line_ending, space0, space1,
    },
    sequence::tuple,
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
    let (input, (_, name, _, _, value, _, _)) = tuple((
        space0,
        parse_header_name,
        tag(":"),
        space0,
        not_line_ending,
        line_ending,
        line_ending,
    ))(input)?;
    Ok((input, (name, value)))
}

pub fn parse_http_request(input: &str) -> IResult<&str, Request> {
    let (input, (method, _, path, _, version, _)) = tuple((
        parse_method,
        space1,
        parse_path,
        space1,
        parse_version,
        multispace0,
    ))(input)?;

    let method = match method.to_uppercase().as_str() {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        "PATCH" => HttpMethod::PATCH,
        "PUT" => HttpMethod::PUT,
        "DELETE" => HttpMethod::DELETE,
        _ => HttpMethod::UNKNOWN,
    };

    let mut headers = HashMap::new();

    let mut rest = input;
    while !rest.is_empty() {
        match parse_header(rest) {
            Ok((new_rest, (name, value))) => {
                headers.insert(name.to_string(), value.trim().to_string());
                rest = new_rest;
                if rest.starts_with("\r\n") {
                    break;
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    let body = if !rest.is_empty() {
        let (_, (_, content, _)) = tuple((multispace0, not_line_ending, multispace0))(rest)?;
        content
    } else {
        ""
    };

    Ok((
        input,
        Request {
            method,
            path: path.replace('\\', "/").to_string(),
            version: version.to_string(),
            headers,
            body: body.to_string(),
        },
    ))
}

#[test]
fn parse_http() {
    let input = "GET /tweets HTTP/1.1\r\n
            User-Agent: PostmanRuntime/7.32.3\r\n
            Accept: */*\r\n
            Cache-Control: no-cache\r\n
            Postman-Token: 953cc42d-60e6-4155-ab58-54d958c62304\r\n
            Host: localhost:4221\r\n
            Accept-Encoding: gzip, deflate, br\r\n
            Connection: keep-alive\r\n
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
