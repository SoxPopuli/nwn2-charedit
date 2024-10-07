use rust_utils::byte_readers::from_bytes_le;
use std::io::{Read, Write};

use crate::error::{Error, IntoError};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ResRef(pub String);

impl ResRef {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let size = {
            let size: u8 = from_bytes_le(&mut data).into_parse_error()?;
            size.clamp(0, 16)
        };

        let data = {
            let mut buf = vec![0u8; size as usize];
            data.read_exact(&mut buf).into_parse_error()?;
            buf
        };

        let s = String::from_utf8(data).into_parse_error()?;

        Ok(Self(s))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let sz = self.0.len() as u8;

        writer.write_all(&[sz]).into_write_error()?;

        let bytes = self.0.as_bytes();
        let len = bytes.len().clamp(0, 16);

        writer.write_all(&bytes[..len]).into_write_error()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_and_write_test() {
        let mut data = Cursor::new([5, b'h', b'e', b'l', b'l', b'o']);

        let r = ResRef::read(&mut data).unwrap();

        assert_eq!(r, ResRef("hello".to_owned()));
        assert_eq!(data.position(), 6);

        let mut output = vec![];
        r.write(&mut output).unwrap();

        assert_eq!(&data.into_inner().as_slice(), &output)
    }
}
