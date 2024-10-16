use crate::error::{Error, IntoError};
use encoding_rs::WINDOWS_1252;
use std::{
    io::{Read, Write},
    sync::Arc,
};

pub(crate) const LABEL_SIZE: usize = 16;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Label(pub(crate) Arc<str>);
impl std::fmt::Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.as_str())
    }
}
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

        let boxed = WINDOWS_1252.decode(slice).0.into();
        Ok(Label(boxed))
    }

    pub fn to_array(&self) -> [u8; LABEL_SIZE] {
        let mut buf = [0u8; LABEL_SIZE];
        let strlen = self.0.len();

        let encoded = WINDOWS_1252.encode(&self.0);

        buf[..strlen].copy_from_slice(&encoded.0);
        buf
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Label {
    type Target = str;
    fn deref(&self) -> &Self::Target {
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
