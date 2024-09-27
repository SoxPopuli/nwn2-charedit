use rust_utils::byte_readers::from_bytes_le;
use rust_utils::string_stream::StringStream;
use std::io::Read;

use crate::error::{Error, IntoParseError};

#[derive(Debug, PartialEq, Eq)]
pub struct ResRef(pub String);

impl ResRef {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let size = {
            let size: u8 = from_bytes_le(&mut data).into_parse_error()?;
            size.clamp(0, 16)
        };
        let stream = StringStream::new(data).take(size as usize);
        let buf = String::from_iter(stream);
        Ok(Self(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_test() {
        let mut data = Cursor::new([5, b'h', b'e', b'l', b'l', b'o']);

        let r = ResRef::read(&mut data).unwrap();

        assert_eq!(r, ResRef("hello".to_owned()));
        assert_eq!(data.position(), 6);
    }
}
