use std::{fmt::Display, io};

#[derive(Debug, Clone)]
pub struct Error {
    kind: Option<io::ErrorKind>,
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Error {
        Error {
            kind: None,
            message: message.to_string(),
        }
    }

    pub fn io_kind(&self) -> Option<io::ErrorKind> {
        self.kind
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error {
            kind: Some(err.kind()),
            message: err.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}
