use std::fmt;
use std::io;
use std::time::SystemTimeError;

/// Application error types
#[derive(Debug)]
pub enum AppError {
    IoError(io::Error),
    FileError(String),
    JsonError(String),
    TotpError(String),
    SystemTimeError(SystemTimeError),
    InvalidInput(String),
    PermissionError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::IoError(e) => write!(f, "IO error: {}", e),
            AppError::FileError(msg) => write!(f, "File error: {}", msg),
            AppError::JsonError(msg) => write!(f, "JSON error: {}", msg),
            AppError::TotpError(msg) => write!(f, "TOTP error: {}", msg),
            AppError::SystemTimeError(e) => write!(f, "System time error: {}", e),
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AppError::PermissionError(msg) => write!(f, "Permission error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        if error.kind() == io::ErrorKind::PermissionDenied {
            AppError::PermissionError(format!("Permission denied: {}", error))
        } else {
            AppError::IoError(error)
        }
    }
}

impl From<SystemTimeError> for AppError {
    fn from(error: SystemTimeError) -> Self {
        AppError::SystemTimeError(error)
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        AppError::FileError(error.to_string())
    }
}

