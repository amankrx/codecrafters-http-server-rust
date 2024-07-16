use std::str::FromStr;

pub const SUCCESS_STATUS: &str = "HTTP/1.1 200 OK";
pub const CREATED_STATUS: &str = "HTTP/1.1 201 Created";
pub const NOT_FOUND_STATUS: &str = "HTTP/1.1 404 Not Found";

#[derive(Default, Debug)]
pub struct Headers {
    pub host: Option<String>,
    pub user_agent: Option<String>,
    pub accept: Option<String>,
    pub content_length: Option<usize>,
    pub accept_encoding: Option<Vec<String>>,
}

#[derive(Default, Debug)]
pub enum RequestMethod {
    #[default]
    Get,
    Post,
}

// NOTE: This can be handled in a much better way with serde or strum.
// But for the sake of this challenge, I'm not using these libraries.
impl FromStr for RequestMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            _ => Err(()),
        }
    }
}

#[derive(Default, Debug)]
pub struct RequestLine {
    pub http_method: RequestMethod,
    pub target: String,
    pub http_version: String,
}

#[derive(Default, Debug)]
pub struct Request {
    pub request_line: RequestLine,
    pub headers: Headers,
    pub body: Option<String>,
}

impl Request {
    pub fn new() -> Self {
        Self::default()
    }
}
