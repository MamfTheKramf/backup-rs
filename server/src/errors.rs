use std::fmt::Display;


#[derive(Debug)]
pub struct Error {
    pub msg: String,
    pub cause: Option<Box<Error>>,
}

#[derive(Debug)]
pub enum ErrorType {
    NotFound(Error),
    UnkownError(Error)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.cause {
            Some(cause) => write!(f, "{:?}: \"{}\"\nCaused by: {}", self.kind, self.msg, cause),
            None => write!(f, "{:?}: \"{}\"", self.kind, self.msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.cause {
            Some(cause) => Some(cause),
            None => None,
        }
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}