use std::error::Error;
use std::io;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Custom { description: String },
    IoError(io::Error)
}

impl AppError {
    pub fn new<S: Into<String>>(text: S) -> AppError {
        AppError::Custom {
            description: text.into()
        }
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> AppError {
        AppError::IoError(err)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppError::Custom { ref description } => write!(f, "{}", description),
            AppError::IoError(ref err) => err.fmt(f),
        }
    }
}

impl Error for AppError {
    fn description(&self) -> &str {
        match *self {
            AppError::Custom { ref description } => description,
            AppError::IoError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            AppError::Custom{..} => None,
            AppError::IoError(ref err) =>  Some(err),
        }
    }
}
