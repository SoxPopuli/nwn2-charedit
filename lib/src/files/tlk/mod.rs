pub mod string_data;

use super::{from_bytes_le, read_string, Language};
use crate::error::Error;
use rust_utils::collect_vec::CollectVecResult;
use std::{
    io::{Read, Seek},
    sync::{Arc, LazyLock},
};

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

static EMPTY_STRING: LazyLock<Arc<str>> = LazyLock::new(|| {
    let s = "";
    Arc::<str>::from(s)
});

pub fn get_empty_string() -> Arc<str> {
    EMPTY_STRING.clone()
}

#[derive(Debug, PartialEq)]
pub struct Tlk {
    pub header: Header,
    pub strings: Vec<Arc<str>>,
}
impl Tlk {
    pub fn read(mut data: impl Read + Seek) -> Result<Self, Error> {
        let header = Header::read(&mut data)?;

        let strings = (0..header.string_count)
            .map(|_| string_data::read(&mut data, header.string_entries_offset as u64))
            .collect_vec_result()?;

        Ok(Self { header, strings })
    }

    pub fn get_from_str_ref(&self, str_ref: u32) -> Option<&Arc<str>> {
        if str_ref == u32::MAX {
            Some(&*EMPTY_STRING)
        } else {
            self.strings.get(str_ref as usize)
        }
    }
}

impl std::ops::Index<u32> for Tlk {
    type Output = str;
    fn index(&self, index: u32) -> &Self::Output {
        self.get_from_str_ref(index)
            .map(|ptr| ptr.as_ref())
            .unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::Tlk;
    use std::io::Cursor;

    #[test]
    #[ignore = "requires proprietary file"]
    fn read_test() {
        use std::time::SystemTime;

        let data = Cursor::new(include_bytes!("../../tests/files/dialog.TLK"));

        let start = SystemTime::now();
        let _tlk = Tlk::read(data).unwrap();
        let end = SystemTime::now();

        let time_to_alloc = end.duration_since(start).unwrap();
        println!("TLK: time to alloc: {:>5}µs", time_to_alloc.as_micros());

        let start = SystemTime::now();
        drop(_tlk);
        let end = SystemTime::now();

        let time_to_drop = end.duration_since(start).unwrap();

        println!("TLK: time to drop:  {:>5}µs", time_to_drop.as_micros());
    }
}
