//! Simple helpers to forward bytes from `Read` to `Write`.

use std::io::{ self, Read, Seek, Write, SeekFrom };

/// Copy bytes from `input` to `output` of the same length as provided `buffer`.
///
/// ## Example
///
/// ```
/// use little::stream;
/// use std::io::Cursor;
///
/// let data: Vec<u8> = "it is interesting".into();
/// let mut input = Cursor::new(data);
/// let mut output = Vec::new();
///
/// let mut buf = [0; 5]; // note the length to copy is equal to buffer len
/// stream::cp(&mut buf, &mut input, &mut output).unwrap();
///
/// assert_eq!("it is", String::from_utf8_lossy(&output[..]));
/// ```
#[inline(always)]
pub fn cp<I, O>(buf: &mut [u8], input: &mut I, output: &mut O)
    -> Result<(), io::Error> where I: Read, O: Write
{
    try!(input.take(buf.len() as u64).read(buf));
    try!(output.write(buf));
    Ok(())
}

/// Copy `len` < `buf.len()` bytes from `input` to `output` using `buf`.
///
/// Buffer must be larger or equal to `len`.
///
/// ## Example
///
/// ```
/// use little::stream;
/// use std::io::Cursor;
///
/// let data: Vec<u8> = "it is interesting".into();
/// let mut input = Cursor::new(data);
/// let mut output = Vec::new();
///
/// let mut buf = [0; 9]; // note the buffer can be larger, but not smaller
/// stream::cp_len(8, &mut buf, &mut input, &mut output).unwrap();
///
/// assert_eq!("it is in", String::from_utf8_lossy(&output[..]));
/// ```
#[inline(always)]
pub fn cp_len<I, O>(len: u64, buf: &mut [u8], input: &mut I, output: &mut O)
    -> Result<(), io::Error> where I: Read, O: Write
{
    try!(input.take(len).read(buf));
    try!(output.write(&buf[..len as usize]));
    Ok(())
}

/// Copy `len` bytes from `input` to `output` using `buf`.
///
/// ## Example
///
/// ```
/// use little::stream;
/// use std::io::Cursor;
///
/// let data: Vec<u8> = "it is interesting".into();
/// let mut input = Cursor::new(data);
/// let mut output = Vec::new();
///
/// let mut buf = [0; 4]; // note the buffer is smaller
/// stream::forward_len(5, &mut buf, &mut input, &mut output).unwrap();
///
/// assert_eq!("it is", String::from_utf8_lossy(&output[..]));
/// ```
pub fn forward_len<I, O>(mut len: u64, buf: &mut [u8], input: &mut I, output: &mut O)
    -> Result<(), io::Error> where I: Read, O: Write
{
    let buf_len = buf.len() as u64;
    loop {
        if len >= buf_len {
            try!(cp(buf, input, output));
            len -= buf_len;
        } else {
            if len == 0 { return Ok(()); }
            try!(cp_len(len, buf, input, output));
            return Ok(());
        }
    }
}

/// Copy `len` bytes from `input` to `output` using `buf` from specified `loc` position.
///
/// ## Example
///
/// ```
/// use little::stream;
/// use std::io::Cursor;
///
/// let data: Vec<u8> = "it is interesting".into();
/// let mut input = Cursor::new(data);
/// let mut output = Vec::new();
///
/// let mut buf = [0; 4]; // note the buffer is smaller
/// stream::seek_and_forward_len(3, 5, &mut buf, &mut input, &mut output).unwrap();
///
/// assert_eq!("is in", String::from_utf8_lossy(&output[..]));
/// ```
#[inline(always)]
pub fn seek_and_forward_len<I, O>(loc: u64, len: u64, buf: &mut [u8], input: &mut I, output: &mut O)
    -> Result<(), io::Error> where I: Read + Seek, O: Write
{
    try!(input.seek(SeekFrom::Start(loc)));
    forward_len(len, buf, input, output)
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

        seek_and_forward_len(6, 5, &mut buf, &mut input, &mut output).unwrap();
        seek_and_forward_len(5, 1, &mut buf, &mut input, &mut output).unwrap();
        seek_and_forward_len(0, 5, &mut buf, &mut input, &mut output).unwrap();
        seek_and_forward_len(5, 1, &mut buf, &mut input, &mut output).unwrap();
        seek_and_forward_len(11, 37, &mut buf, &mut input, &mut output).unwrap();

        assert_eq!(
            "hello world and the quick fox jumps over that dog",
            String::from_utf8_lossy(&output[..])
        );
    }
}

#[cfg(feature="nightly")]
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
            seek_and_forward_len(6, 5, &mut buf, &mut input, &mut output).unwrap();
            seek_and_forward_len(5, 1, &mut buf, &mut input, &mut output).unwrap();
            seek_and_forward_len(0, 5, &mut buf, &mut input, &mut output).unwrap();
            seek_and_forward_len(5, 1, &mut buf, &mut input, &mut output).unwrap();
            seek_and_forward_len(11, 37, &mut buf, &mut input, &mut output).unwrap();

            output
        });
    }
}
