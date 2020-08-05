use std::{fmt, string};

#[derive(Debug)]
pub struct ParseErr {
    pub reason: String,
}

impl From<string::FromUtf8Error> for ParseErr {
    fn from(error: std::string::FromUtf8Error) -> Self {
        ParseErr {
            reason: String::from("malformed data"),
        }
    }
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tokenize error: {}", self.reason)
    }
}

impl std::error::Error for ParseErr {}
