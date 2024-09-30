use rust_utils::collect_vec::CollectVecResult;

use crate::{
    error::{Error, IntoError},
    files::{from_bytes_le, tlk::Tlk, Gender, Language},
};
use std::{
    io::{Read, Write},
    sync::Arc,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ExoString(pub String);
impl ExoString {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let size: u32 = from_bytes_le(&mut data).into_parse_error()?;

        let buf = {
            let mut buf = vec![0u8; size as usize];
            data.read_exact(&mut buf).into_parse_error()?;
            buf
        };

        let str = String::from_utf8(buf).into_parse_error()?;

        Ok(Self(str))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let sz = self.0.len() as u32;

        writer.write_all(&sz.to_le_bytes()).into_write_error()?;
        writer.write_all(self.0.as_bytes()).into_write_error()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExoLocString {
    tlk_string: Arc<str>,
    substrings: Vec<ExoLocSubString>,
}
impl ExoLocString {
    pub fn read(mut data: impl Read, tlk: &Tlk) -> Result<Self, Error> {
        let _size = from_bytes_le::<u32>(&mut data)? as usize;
        let str_ref: u32 = from_bytes_le(&mut data)?;
        let str_count: u32 = from_bytes_le(&mut data)?;

        let tlk_string = if str_ref == u32::MAX {
            crate::files::tlk::get_empty_string()
        } else {
            tlk.get_from_str_ref(str_ref as u32)
                .unwrap_or_else(|| panic!("No string found for ref: {str_ref}"))
                .clone()
        };

        let substrings = (0..str_count)
            .map(|_| ExoLocSubString::read(&mut data))
            .collect_vec_result()?;

        Ok(Self {
            tlk_string,
            substrings,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ExoLocSubString {
    pub gender: Gender,
    pub language: Language,
    pub data: String,
}
impl ExoLocSubString {
    fn read(mut data: impl Read) -> Result<Self, Error> {
        let string_id: i32 = from_bytes_le(&mut data)?;
        let string_length: i32 = from_bytes_le(&mut data)?;

        let gender = Gender::try_from((string_id & 1) as u8)?;
        let language = {
            let id = string_id & (!1);
            Language::try_from((id / 2) as u8)
        }?;

        let s = {
            let mut buf = vec![0u8; string_length as usize];
            data.read_exact(&mut buf).into_parse_error()?;
            String::from_utf8_lossy(&buf).to_string()
        };

        Ok(Self {
            gender,
            language,
            data: s,
        })
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
