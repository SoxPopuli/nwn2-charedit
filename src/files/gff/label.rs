use std::io::Read;

use crate::error::{Error, IntoParseError};

const LABEL_SIZE: usize = 16;

#[derive(Debug)]
pub struct Label(pub [u8; LABEL_SIZE]);
impl Label {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let mut buf = [0u8; LABEL_SIZE];
        data.read_exact(&mut buf).into_parse_error()?;

        Ok(Label(buf))
    }

    pub fn get_string(&self) -> Result<&str, Error> {
        let str_end = self.0.into_iter().position(|x| x == b'\0');

        let slice = if let Some(end) = str_end {
            &self.0[..end]
        } else {
            &self.0
        };

        std::str::from_utf8(slice).into_parse_error()
    }

    /// # Safety
    ///
    /// Data must be valid UTF-8
    pub unsafe fn get_string_unchecked(&self) -> &str {
        let str_end = self.0.into_iter().position(|x| x == b'\0');

        let slice = if let Some(end) = str_end {
            &self.0[..end]
        } else {
            &self.0
        };

        std::str::from_utf8_unchecked(slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailing_zero_test() {
        let trailing_zeros = [
            b'h', b'e', b'l', b'l', b'o', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let label = Label(trailing_zeros);

        unsafe {
            assert_eq!(label.get_string(), Ok("hello"));
            assert_eq!(label.get_string_unchecked(), "hello");
        }
    }

    #[test]
    fn empty_test() {
        let empty = [0u8; LABEL_SIZE];
        let label = Label(empty);

        assert_eq!(label.get_string(), Ok(""));
    }

    #[test]
    fn full_test() {
        let full = [b'a'; LABEL_SIZE];
        let label = Label(full);

        assert_eq!(label.get_string(), Ok("aaaaaaaaaaaaaaaa"));
    }
}
