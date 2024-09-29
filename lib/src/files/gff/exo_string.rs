use crate::error::{Error, IntoError};
use rust_utils::byte_readers::{from_bytes_le, FromBytes};

use std::io::{Read, Write};

#[derive(Debug, PartialEq, Eq)]
pub struct ExoString(pub String);
impl ExoString {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let size: i32 = from_bytes_le(&mut data).into_parse_error()?;

        let buf = {
            let mut buf = vec![0u8; size as usize];
            data.read_exact(&mut buf).into_parse_error()?;
            buf
        };

        let str = String::from_utf8(buf).into_parse_error()?;

        Ok(Self(str))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let sz = self.0.len() as i32;

        writer.write_all(&sz.to_le_bytes()).into_write_error()?;
        writer.write_all(self.0.as_bytes()).into_write_error()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExoLocString();
impl ExoLocString {
    pub fn read(mut data: impl Read) -> Result<(), Error> {
        let size = i32::from_bytes_le(&mut data).into_parse_error()? as usize;
        let str_ref = i32::from_bytes_le(&mut data).into_parse_error()?;
        let str_count = i32::from_bytes_le(&mut data).into_parse_error()?;

        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_and_write_test() {
        let data = Cursor::new([0x04, 0x00, 0x00, 0x00, b'T', b'e', b's', b't']);

        let x = ExoString::read(data.clone()).unwrap();
        assert_eq!(x, ExoString("Test".to_string()));

        let mut output = Cursor::new(vec![]);
        x.write(&mut output).unwrap();
        assert_eq!(output.into_inner().as_slice(), &data.into_inner())
    }
}
