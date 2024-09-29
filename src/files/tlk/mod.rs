pub mod string_data;

use super::{from_bytes_le, read_string, Language};
use crate::error::Error;
use rust_utils::collect_vec::CollectVecResult;
use std::io::{Read, Seek};

#[derive(Debug, PartialEq)]
pub struct Header {
    file_type: String,
    file_version: f32,
    language: Language,
    string_count: u32,
    string_entries_offset: u32,
}
impl Header {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let file_type = read_string(&mut data, 4)?;

        let file_version = {
            let str = read_string(&mut data, 4)?;
            (str[1..]).trim_end().parse()
        }?;

        let language: Language = {
            let lang: u32 = from_bytes_le(&mut data)?;
            (lang as u8).try_into()
        }?;

        let string_count: u32 = from_bytes_le(&mut data)?;

        let string_entries_offset: u32 = from_bytes_le(&mut data)?;

        Ok(Self {
            file_type,
            file_version,
            language,
            string_count,
            string_entries_offset,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Tlk {
    pub header: Header,
    pub strings: Vec<String>,
}
impl Tlk {
    pub fn read(mut data: impl Read + Seek) -> Result<Self, Error> {
        let header = Header::read(&mut data)?;

        let strings = (0..header.string_count)
            .map(|_| string_data::read(&mut data, header.string_entries_offset as u64))
            .collect_vec_result()?;

        Ok(Self { header, strings })
    }
}

#[cfg(test)]
mod tests {
    use super::Tlk;
    use std::io::Cursor;

    #[test]
    fn x() {
        let data = Cursor::new(include_bytes!("../../../../../Downloads/dialog.TLK"));

        let _tlk = Tlk::read(data).unwrap();
    }
}
