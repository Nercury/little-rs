use std::io;
use std::error;
use std::fmt;

/// Error while performing seek.
#[derive(Debug)]
pub enum BuildError {
    /// Out of bound operation on container.
    FunctionNotFound { required: String },
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BuildError::FunctionNotFound { ref required } => write!(f, "Function {:?} not found", required),
        }
    }
}

impl error::Error for BuildError {
    fn description(&self) -> &str {
        match *self {
            BuildError::FunctionNotFound { .. } => "function not found",
        }
    }
}
