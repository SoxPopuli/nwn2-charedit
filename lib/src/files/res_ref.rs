use super::{from_bytes_le, gff::Writeable};
use crate::error::{Error, IntoError};
use encoding_rs::WINDOWS_1252;
use std::io::{Read, Write};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ResRef(pub String);

impl ResRef {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let size = from_bytes_le::<u8>(&mut data)?;

        let data = {
            let mut buf = vec![0u8; size as usize];
            data.read_exact(&mut buf).into_parse_error()?;
            buf
        };

        let s =
            // String::from_utf8(data).into_parse_error()?;
            WINDOWS_1252.decode(&data).0.to_string();

        Ok(Self(s))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let sz = self.0.len() as u8;

        writer.write_all(&sz.to_le_bytes()).into_write_error()?;

        let data = WINDOWS_1252.encode(&self.0).0;
        let len = data.len();
        let data = &data[..len];

        writer.write_all(data).into_write_error()?;

        Ok(())
    }
}
impl Writeable for &ResRef {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        ResRef::write(self, writer)
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
