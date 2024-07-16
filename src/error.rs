#[derive(Debug)]
pub enum Error {
    ParseError(String),
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::ParseError(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::ParseError(format!("IO Error: {}", err))
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::ParseError(format!("UTF-8 Error: {}", err))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(msg) => write!(f, "Parse Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
