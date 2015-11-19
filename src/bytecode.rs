/*!
Bytecode `io` helpers.
*/

use std::io;
use std::mem;
use byteorder::{ self, LittleEndian, ReadBytesExt, WriteBytesExt };

/// Bytecode representation.
pub trait Bytecode {
    
}

/// Serialize and deserialize a structure from `io`.
pub trait Serializer {
    /// Write contents to io `writer`, returns bytes written.
    fn serialize<O: io::Write>(&self, writer: &mut O) -> Result<u64, Error>;
    /// Read contents from `reader`, return tuple of bytes read and new structure.
    fn deserialize<I: io::Read>(reader: &mut I) -> Result<(u64, Self), Error> where Self: Sized;
}

/// Bytecode read/write error.
#[derive(Debug)]
pub enum Error {
    /// Failed to read cache header, assume this is not valid cache file.
    InvalidBinaryFormat,
    UnexpectedEOF,
    Io(io::Error),
}

impl From<byteorder::Error> for Error {
    fn from(other: byteorder::Error) -> Error {
        match other {
            byteorder::Error::UnexpectedEOF => Error::UnexpectedEOF,
            byteorder::Error::Io(e) => Error::Io(e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Error {
        Error::Io(other)
    }
}

/// Bytecode file header.
#[derive(Eq, PartialEq, Debug)]
pub struct Header {
    magic: u32,
}

impl Header {
    pub fn new() -> Header {
        Header {
            magic: Header::magic(),
        }
    }

    /// Check if header is valid.
    pub fn is_magical(&self) -> bool {
        self.magic == Header::magic()
    }

    /// Return magic header number.
    fn magic() -> u32 {
        52231103
    }
}

impl Serializer for Header {
    fn serialize<O: io::Write>(&self, output: &mut O) -> Result<u64, Error> {
        try!(output.write_u32::<LittleEndian>(self.magic));
        Ok(mem::size_of::<Header>() as u64)
    }

    fn deserialize<I: io::Read>(input: &mut I) -> Result<(u64, Header), Error> {
        Ok((mem::size_of::<Header>() as u64, Header {
            magic: try!(input.read_u32::<LittleEndian>())
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn header() {
        let mut input: Vec<u8> = vec![];

        let a = Header::new();
        a.serialize(&mut input).unwrap();

        let mut cursor = Cursor::new(&input[..]);
        let (_, b) = Header::deserialize(&mut cursor).unwrap();
        assert_eq!(a, b);
    }

}
