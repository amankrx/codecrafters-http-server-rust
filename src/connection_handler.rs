use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::{env, fs};
use nom::AsBytes;

use crate::error::Error;
use crate::models::{Request, RequestMethod, CREATED_STATUS, NOT_FOUND_STATUS, SUCCESS_STATUS};

fn parse_request(mut buf_reader: BufReader<&mut TcpStream>) -> Result<Request, Error> {
    let mut request = Request::new();
    let mut buffer_lines = buf_reader.by_ref().lines();

    // Parse Request Line
    if let Some(line) = buffer_lines.next() {
        let line = line?;
        let mut parts = line.split_whitespace();

        request.request_line.http_method = parts
            .next()
            .ok_or_else(|| Error::ParseError("Failed to parse HTTP method".to_string()))?
            .parse()
            .map_err(|_| Error::ParseError("Failed to parse HTTP method".to_string()))?;

        request.request_line.target = parts
            .next()
            .ok_or_else(|| Error::ParseError("Failed to parse request target".to_string()))?
            .to_string();

        request.request_line.http_version = parts
            .next()
            .ok_or_else(|| Error::ParseError("Failed to parse HTTP version".to_string()))?
            .to_string();
    }

    // Parse Request Headers
    for line in buffer_lines {
        match line {
            Ok(line) if line.is_empty() => break, // End of headers
            Ok(line) => {
                if let Some((key, value)) = line.split_once(": ") {
                    match key {
                        "Host" => request.headers.host = Some(value.to_string()),
                        "User-Agent" => request.headers.user_agent = Some(value.to_string()),
                        "Accept" => request.headers.accept = Some(value.to_string()),
                        "Accept-Encoding" => {
                            request.headers.accept_encoding =
                                Some(value.split(", ").map(|s| s.to_string()).collect())
                        }
                        "Content-Length" => {
                            request.headers.content_length =
                                Some(value.trim().parse().map_err(|_| {
                                    Error::ParseError("Failed to parse Content-Length".to_string())
                                })?)
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => return Err(Error::ParseError(e.to_string())),
        }
    }

    // Parse Request Body
    if let Some(len) = request.headers.content_length {
        if len > 0 {
            let mut buf = vec![0; len];
            buf_reader.read_exact(&mut buf)?;
            request.body = Some(String::from_utf8(buf)?);
        }
    }

    Ok(request)
}

fn handle_get_response(request: Request) -> Result<Vec<u8>, Error> {
    match request.request_line.target.as_str() {
        "/" => Ok(Vec::from(format!("{}\r\n\r\n", SUCCESS_STATUS).as_bytes())),
        "/user-agent" => match &request.headers.user_agent {
            Some(user_agent) => Ok(Vec::from(
                format!(
                    "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    SUCCESS_STATUS,
                    user_agent.len(),
                    user_agent
                )
                .as_bytes(),
            )),
            None => Ok(Vec::from(format!("{}\r\n\r\n", SUCCESS_STATUS).as_bytes())),
        },
        file if file.starts_with("/files/") => {
            let filename = file["/files/".len()..].to_string();
            let env_args: Vec<String> = env::args().collect();
            let mut dir = env_args[2].clone();
            dir.push_str(&filename);
            let file = fs::read(dir);

            match file {
                Ok(res) => {
                    Ok(Vec::from(format!(
                        "{}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                        SUCCESS_STATUS,
                        res.len(),
                        String::from_utf8(res).expect("file content")
                    ).as_bytes()))
                },
                Err(_) => {
                    Ok(Vec::from(format!("{}\r\n\r\n", NOT_FOUND_STATUS).as_bytes()))
                }
            }
        }
        path if path.starts_with("/echo/") => {
            let response_body = path["/echo/".len()..].to_string();
            match request.headers.accept_encoding {
                Some(accept_encoding) => {
                    if accept_encoding.contains(&"gzip".to_string()) {
                        let mut gz_encoder = GzEncoder::new(Vec::new(), Compression::default());
                        gz_encoder.write_all(response_body.as_bytes())?;

                        let compressed = gz_encoder.finish()?;
                        let content_length = compressed.len();
                        let response = format!(
                            "{}\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n",
                            SUCCESS_STATUS,
                            content_length
                        );
                        Ok([response.as_bytes(), &compressed].concat())
                    } else {
                        Ok(Vec::from(
                            format!("{}\r\nContent-Type: text/plain\r\n\r\n", SUCCESS_STATUS)
                                .as_bytes(),
                        ))
                    }
                }
                None => Ok(Vec::from(
                    format!(
                        "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        SUCCESS_STATUS,
                        response_body.len(),
                        response_body
                    )
                    .as_bytes(),
                )),
            }
        }
        _ => Ok(Vec::from(
            format!("{}\r\n\r\n", NOT_FOUND_STATUS).as_bytes(),
        )),
    }
}

fn handle_post_response(request: Request) -> Result<Vec<u8>, Error> {
    match request.request_line.target.as_str() {
        file if file.starts_with("/files/") => {
            let filename = file["/files/".len()..].to_string();
            let env_args: Vec<String> = env::args().collect();
            let mut dir = env_args[2].clone();
            if !Path::new(&dir).exists() {
                fs::create_dir_all(&dir).expect("Failed to create directory");
            }
            dir.push_str(&filename);
            fs::write(&dir, request.body.unwrap()).expect("Unable to write file");
            Ok(Vec::from(format!("{}\r\n\r\n", CREATED_STATUS).as_bytes()))
        }
        _ => Ok(Vec::from(
            format!("{}\r\n\r\n", NOT_FOUND_STATUS).as_bytes(),
        )),
    }
}

pub fn handle_connection(mut stream: TcpStream) -> Result<(), Error> {
    let buf_reader = BufReader::new(&mut stream);
    let request = parse_request(buf_reader).unwrap();
    let response = match request.request_line.http_method {
        RequestMethod::Get => handle_get_response(request),
        RequestMethod::Post => handle_post_response(request),
    }?;

    stream.write_all(response.as_bytes())?;
    Ok(())
}
