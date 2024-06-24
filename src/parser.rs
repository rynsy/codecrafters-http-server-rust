use crate::types::Request;

use nom::{
    bytes::complete::{tag, take_while},
    character::complete::{alphanumeric1, multispace0, newline, space1},
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
    take_while(|c: char| is_token_char(c) || c == '/' || c == '?')(input)
}

fn parse_version(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("HTTP/")(input)?;
    take_while(|c: char| c.is_digit(10) || c == '.')(input)
}
fn parse_user_agent(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("User-Agent: ")(input)?;
    take_while(|c: char| c != '\n')(input)
}

pub fn parse_http_request(input: &str) -> IResult<&str, Request> {
    let (input, (method, _, path, _, version, _, user_agent)) = tuple((
        parse_method,
        space1,
        parse_path,
        space1,
        parse_version,
        multispace0,
        parse_user_agent,
    ))(input)?;

    Ok((
        input,
        Request {
            method: method.to_string(),
            path: path.replace('\\', "/").to_string(),
            version: version.to_string(),
            user_agent: user_agent.to_string(),
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
            assert_eq!(result.user_agent, "PostmanRuntime/7.32.3");
        }
        Err(e) => {
            eprintln!("Error! {:?}", e);
            assert_eq!(true, false);
        }
    }
}
