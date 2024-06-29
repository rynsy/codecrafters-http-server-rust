use crate::types::Request;
use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_while},
    character::complete::{alphanumeric1, multispace0, not_line_ending, space0, space1},
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
    let (input, (name, _, _, value, _)) = tuple((
        parse_header_name,
        tag(":"),
        space0,
        not_line_ending,
        multispace0,
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

    let mut headers = HashMap::new();
    let mut rest = input;

    while !rest.is_empty() {
        let (new_rest, (name, value)) = parse_header(rest)?;
        headers.insert(name.to_string(), value.trim().to_string());
        rest = new_rest;
        if rest.starts_with("\r\n") {
            break;
        }
    }

    Ok((
        input,
        Request {
            method: method.to_string(),
            path: path.replace('\\', "/").to_string(),
            version: version.to_string(),
            headers,
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
            Connection: keep-alive";

    match parse_http_request(input) {
        Ok((_, result)) => {
            assert_eq!(result.method, "GET");
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
        }
        Err(e) => {
            eprintln!("Error! {:?}", e);
            assert_eq!(true, false);
        }
    }
}
