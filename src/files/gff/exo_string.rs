use crate::error::{Error, IntoParseError};
use rust_utils::{byte_readers::from_bytes_le, string_stream::StringStream};
use std::io::Read;

#[derive(Debug, PartialEq, Eq)]
pub struct ExoString(pub String);
impl ExoString {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let size: i32 = from_bytes_le(&mut data).into_parse_error()?;

        let stream = StringStream::new(data).take(size as usize);

        Ok(Self(String::from_iter(stream)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Seek, Write};

    #[test]
    fn read_test() {
        let mut data = Cursor::new(Vec::<u8>::new());

        let str_data = [b'T', b'e', b's', b't'];

        data.write_all(&(str_data.len() as i32).to_le_bytes())
            .unwrap();
        data.write_all(&str_data).unwrap();

        data.rewind().unwrap();

        let x = ExoString::read(data).unwrap();
        assert_eq!(x, ExoString("Test".to_string()));
    }
}
