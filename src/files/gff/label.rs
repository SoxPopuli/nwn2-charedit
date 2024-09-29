use std::io::{Read, Write};

use crate::error::{Error, IntoError};

const LABEL_SIZE: usize = 16;

#[derive(Debug, PartialEq, Eq)]
pub struct Label(pub String);
impl Label {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let mut buf = [0u8; LABEL_SIZE];
        data.read_exact(&mut buf).into_parse_error()?;

        Self::new(buf)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.to_array()).into_write_error()
    }

    pub fn new(data: [u8; LABEL_SIZE]) -> Result<Self, Error> {
        let strend = data.into_iter().position(|x| x == 0);
        let slice = match strend {
            Some(end) => &data[..end],
            None => &data,
        };

        let s = std::str::from_utf8(slice).into_parse_error()?;
        Ok(Label(s.to_string()))
    }

    pub fn to_array(&self) -> [u8; LABEL_SIZE] {
        let mut buf = [0u8; LABEL_SIZE];
        let strlen = self.0.len();

        buf[..strlen].copy_from_slice(self.0.as_bytes());
        buf
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialEq<&str> for Label {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn trailing_zero_test() {
        let trailing_zeros = [
            b'h', b'e', b'l', b'l', b'o', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let label = Label::new(trailing_zeros).unwrap();

        assert_eq!(label, "hello");
    }

    #[test]
    fn empty_test() {
        let empty = [0u8; LABEL_SIZE];
        let label = Label::new(empty).unwrap();

        assert_eq!(label, "");
    }

    #[test]
    fn full_test() {
        let full = [b'a'; LABEL_SIZE];
        let label = Label::new(full).unwrap();

        assert_eq!(label, "aaaaaaaaaaaaaaaa");
    }

    #[test]
    fn read_and_write_test() {
        let data = [b'h', b'i', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let label = Label::new(data).unwrap();

        assert_eq!(label, "hi");

        let mut buf = Cursor::new(vec![]);
        label.write(&mut buf).unwrap();

        assert_eq!(buf.into_inner(), data,)
    }
}
