//! Simple helpers to forward bytes from `Read` to `Write`.

use std::io::{ self, Read, Seek, Write, SeekFrom, ErrorKind };

/// Copy all bytes from `reader` to `writer` using `buf`.
///
/// ## Example
///
/// ```
/// use little::stream;
/// use std::io::{ Cursor, Read };
///
/// let data: &[u8] = b"it is interesting";
/// let mut reader = Cursor::new(data);
/// let mut writer = Vec::new();
///
/// let mut buf = [0; 4]; // note the buffer is smaller
/// stream::buf_copy(&mut buf, &mut reader.take(5), &mut writer).unwrap();
///
/// assert_eq!("it is", String::from_utf8_lossy(&writer[..]));
/// ```
pub fn buf_copy<I, O>(buf: &mut [u8], reader: &mut I, writer: &mut O)
    -> Result<u64, io::Error> where I: Read, O: Write
{
    let mut written = 0;
    loop {
        let len = match reader.read(buf) {
            Ok(0) => return Ok(written),
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        try!(writer.write_all(&buf[..len]));
        written += len as u64;
    }
}

/// Copy `len` bytes from `reader` to `writer` using `buf` from specified `loc` position.
///
/// ## Example
///
/// ```
/// use little::stream;
/// use std::io::Cursor;
///
/// let data: &[u8] = b"it is interesting";
/// let mut reader = Cursor::new(data);
/// let mut writer = Vec::new();
///
/// let mut buf = [0; 4]; // note the buffer is smaller
/// stream::seek_and_buf_copy(3, 5, &mut buf, &mut reader, &mut writer).unwrap();
///
/// assert_eq!("is in", String::from_utf8_lossy(&writer[..]));
/// ```
#[inline(always)]
pub fn seek_and_buf_copy<I, O>(loc: u64, len: u64, buf: &mut [u8], input: &mut I, output: &mut O)
    -> Result<u64, io::Error> where I: Read + Seek, O: Write
{
    try!(input.seek(SeekFrom::Start(loc)));
    buf_copy(buf, &mut input.take(len), output)
}

#[cfg(test)]
mod test {
    use std::io::{ Cursor };
    use super::*;

    #[test]
    fn test_stream() {
        let data = b"world helloand the quick fox jumps over that dog";
        let mut input = Cursor::new(&data[..]);
        let mut output = Vec::<u8>::new();

        let mut buf = [0; 5];

        seek_and_buf_copy(6, 5, &mut buf, &mut input, &mut output).unwrap();
        seek_and_buf_copy(5, 1, &mut buf, &mut input, &mut output).unwrap();
        seek_and_buf_copy(0, 5, &mut buf, &mut input, &mut output).unwrap();
        seek_and_buf_copy(5, 1, &mut buf, &mut input, &mut output).unwrap();
        seek_and_buf_copy(11, 37, &mut buf, &mut input, &mut output).unwrap();

        assert_eq!(
            "hello world and the quick fox jumps over that dog",
            String::from_utf8_lossy(&output[..])
        );
    }
}

#[cfg(bench)]
mod bench {
    extern crate test;

    use std::io::{ Cursor };
    use super::*;

    #[bench]
    fn bench_all_streaming(b: &mut test::Bencher) {
        let data = b"world helloand the quick fox jumps over that dog";

        b.iter(|| {
            let mut input = test::black_box(Cursor::new(&data[..]));
            let mut output = Vec::<u8>::new();

            let mut buf = [0; 5];
            seek_and_buf_copy(6, 5, &mut buf, &mut input, &mut output).unwrap();
            seek_and_buf_copy(5, 1, &mut buf, &mut input, &mut output).unwrap();
            seek_and_buf_copy(0, 5, &mut buf, &mut input, &mut output).unwrap();
            seek_and_buf_copy(5, 1, &mut buf, &mut input, &mut output).unwrap();
            seek_and_buf_copy(11, 37, &mut buf, &mut input, &mut output).unwrap();

            output
        });
    }
}
