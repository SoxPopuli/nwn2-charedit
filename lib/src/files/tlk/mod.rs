pub mod reader;

use super::{Language, Offset, from_bytes_le, offset::ToOffset, read_string};
use crate::error::Error;
use reader::{StringInfo, TlkReader};
use rust_utils::collect_vec::CollectVecResult;
use std::{
    io::{Cursor, Read, Seek},
    sync::{Arc, LazyLock},
};

#[derive(Debug, Default, PartialEq)]
pub struct Header {
    file_type: String,
    file_version: f32,
    language: Language,
    string_count: u32,
    string_entry_offset: u32,
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
            string_entry_offset: string_entries_offset,
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
pub struct Tlk<R: Read + Seek = Cursor<Vec<u8>>> {
    pub header: Header,
    pub reader: TlkReader<R>,
}
impl<R> Default for Tlk<R>
where
    R: Read + Seek + Default,
{
    fn default() -> Self {
        Self {
            header: Default::default(),
            reader: TlkReader::default(),
        }
    }
}
impl<R: Read + Seek> Tlk<R> {
    pub fn read(mut data: R) -> Result<Self, Error> {
        let header = Header::read(&mut data)?;

        let string_info = (0..header.string_count)
            .map(|_| StringInfo::read(&mut data))
            .collect_vec_result()?;

        let reader = TlkReader::new(string_info, header.string_entry_offset.to_offset(), data);

        Ok(Self { header, reader })
    }

    pub fn get_from_str_ref(&self, str_ref: u32) -> Result<Arc<str>, Error> {
        if str_ref == u32::MAX {
            Ok(EMPTY_STRING.clone())
        } else {
            self.reader.read_index(str_ref)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tlk;
    use std::io::Cursor;

    #[test]
    fn read_test() {
        use std::time::SystemTime;

        let data = Cursor::new(include_bytes!("../../tests/files/dialog.TLK"));

        let start = SystemTime::now();
        let tlk = Tlk::read(data).unwrap();
        let end = SystemTime::now();

        let time_to_alloc = end.duration_since(start).unwrap();
        println!("TLK: time to alloc: {:>5}ms", time_to_alloc.as_millis());

        let strings = (0..100).map(|i| tlk.get_from_str_ref(i).unwrap());

        for s in strings {
            println!("{s}");
        }

        let start = SystemTime::now();
        drop(tlk);
        let end = SystemTime::now();

        let time_to_drop = end.duration_since(start).unwrap();

        println!("TLK: time to drop:  {:>5}ms", time_to_drop.as_millis());
    }
}
