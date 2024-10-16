use super::Writeable;
use crate::{
    error::{Error, IntoError},
    files::{from_bytes_le, tlk::Tlk, write_all, Gender, Language},
};
use rust_utils::collect_vec::CollectVecResult;
use std::{
    io::{Read, Seek, Write},
    sync::Arc,
};
use encoding_rs::WINDOWS_1252;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ExoString(pub String);
impl ExoString {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let size: u32 = from_bytes_le(&mut data).into_parse_error()?;

        let buf = {
            let mut buf = vec![0u8; size as usize];
            data.read_exact(&mut buf).into_parse_error()?;
            buf
        };

        let str = 
            // String::from_utf8(buf).into_parse_error()?;
            WINDOWS_1252.decode(&buf).0.to_string();

        Ok(Self(str))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let sz = self.0.len() as u32;
        writer.write_all(&sz.to_le_bytes()).into_write_error()?;

        let data = WINDOWS_1252.encode(&self.0).0;

        writer.write_all(&data).into_write_error()
    }
}
impl Writeable for &ExoString {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        ExoString::write(self, writer)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExoLocString {
    str_ref: u32,
    tlk_string: Option<Arc<str>>,
    substrings: Vec<ExoLocSubString>,
}
impl ExoLocString {
    pub fn read<R>(mut data: impl Read, tlk: &Tlk<R>) -> Result<Self, Error>
    where
        R: Read + Seek,
    {
        let _size = from_bytes_le::<u32>(&mut data)? as usize;
        let str_ref: u32 = from_bytes_le(&mut data)?;
        let str_count: u32 = from_bytes_le(&mut data)?;

        let tlk_string = if str_ref == u32::MAX {
            None
        } else {
            Some(tlk.get_from_str_ref(str_ref as u32)?.clone())
        };

        let substrings = (0..str_count)
            .map(|_| ExoLocSubString::read(&mut data))
            .collect_vec_result()?;

        assert_eq!(
            _size as u32,
            Self::get_total_size(&substrings),
            "Computed size does not match existing"
        );

        Ok(Self {
            str_ref,
            tlk_string,
            substrings,
        })
    }

    fn get_total_size(substrings: &[ExoLocSubString]) -> u32 {
        let substrings_size: u32 = substrings.iter().map(|s| s.get_file_data_size()).sum();
        substrings_size + 8
    }

    pub fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        let total_size = Self::get_total_size(&self.substrings);

        write_all(writer, &total_size.to_le_bytes())?;
        write_all(writer, &self.str_ref.to_le_bytes())?;

        let string_count = self.substrings.len() as u32;
        write_all(writer, &string_count.to_le_bytes())?;

        for s in &self.substrings {
            s.write(writer)?;
        }

        Ok(())
    }
}
impl Writeable for &ExoLocString {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        ExoLocString::write(self, writer)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ExoLocSubString {
    pub gender: Gender,
    pub language: Language,
    pub data: String,
}
impl ExoLocSubString {
    fn get_file_data_size(&self) -> u32 {
        self.data.len() as u32 + 8
    }

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
            WINDOWS_1252.decode(&buf).0.to_string()
        };

        Ok(Self {
            gender,
            language,
            data: s,
        })
    }

    pub fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        let string_id = {
            let language = (self.language.as_u8() as u32) * 2;
            let gender = self.gender.as_u8() as u32;

            language + gender
        };
        let string_length = self.data.len() as u32;

        write_all(writer, &string_id.to_le_bytes())?;
        write_all(writer, &string_length.to_le_bytes())?;

        let data = WINDOWS_1252.encode(&self.data).0;
        write_all(writer, &data)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn exo_read_and_write_test() {
        let data = Cursor::new([0x04, 0x00, 0x00, 0x00, b'T', b'e', b's', b't']);

        let x = ExoString::read(data.clone()).unwrap();
        assert_eq!(x, ExoString("Test".to_string()));

        let mut output = Cursor::new(vec![]);
        x.write(&mut output).unwrap();
        assert_eq!(output.into_inner().as_slice(), &data.into_inner())
    }

    #[test]
    fn exo_loc_read_and_write_test() {
        let str = ExoLocString {
            str_ref: u32::MAX,
            tlk_string: None,
            substrings: vec![ExoLocSubString {
                gender: Gender::Masculine,
                language: Language::English,
                data: "Hello".to_string(),
            }],
        };

        let mut buf = Cursor::new(vec![]);
        str.write(&mut buf).unwrap();
        buf.rewind().unwrap();

        let tlk: Tlk<Cursor<Vec<u8>>> = Tlk::default();
        let str_2 = ExoLocString::read(&mut buf, &tlk).unwrap();

        assert_eq!(str, str_2)
    }
}
