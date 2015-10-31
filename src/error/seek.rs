use std::io;
use std::error;
use std::fmt;

/// Error while performing seek.
#[derive(Debug)]
pub enum SeekError {
    /// Out of bound operation on container.
    OutOfBounds(u32),
    /// I/O error in underlying stream.
    Io(io::Error),
}

impl fmt::Display for SeekError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SeekError::OutOfBounds(v) => write!(f, "Position out of bounds {:?}", v),
            SeekError::Io(ref v) => fmt::Display::fmt(v, f),
        }
    }
}

impl error::Error for SeekError {
    fn description(&self) -> &str {
        match *self {
            SeekError::OutOfBounds(_) => "out of bounds",
            SeekError::Io(ref e) => error::Error::description(e),
        }
    }
}
