use super::{from_bytes_le, Offset};
use crate::error::{Error, IntoError};
use std::{
    collections::{hash_map::Entry, HashMap},
    io::{Read, Seek},
    sync::Arc,
};

#[derive(Debug, PartialEq, Eq)]
pub struct StringInfo {
    pub(crate) offset: Offset,
    pub(crate) size: u32,
}
impl StringInfo {
    pub fn read(mut data: impl Read + Seek) -> Result<Self, Error> {
        data.seek_relative(28).into_parse_error()?;

        let offset_to_string = Offset(from_bytes_le(&mut data)?);
        let string_size: u32 = from_bytes_le(&mut data)?;

        data.seek_relative(4).into_parse_error()?;

        Ok(Self {
            offset: offset_to_string,
            size: string_size,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TlkReader<R>
where
    R: Read + Seek,
{
    pub(crate) data: R,
    pub(crate) string_info: Vec<StringInfo>,
    pub(crate) string_entry_offset: Offset,
    pub(crate) entry_cache: HashMap<u32, Arc<str>>,
}

fn read_str(mut data: impl Read, strlen: usize) -> Result<Arc<str>, Error> {
    let mut buf = vec![0u8; strlen];

    data.read_exact(&mut buf).into_parse_error()?;

    let x = String::from_utf8_lossy(&buf);
    Ok(x.into())
}

impl<R> TlkReader<R>
where
    R: Read + Seek,
{
    pub(crate) fn read_index(&mut self, index: u32) -> Result<Arc<str>, Error> {
        match self.entry_cache.entry(index) {
            Entry::Vacant(e) => {
                let info = self
                    .string_info
                    .get(index as usize)
                    .ok_or(Error::InvalidStrRef { value: index })?;

                info.offset
                    .seek_with_offset(&mut self.data, self.string_entry_offset)?;

                let str = read_str(&mut self.data, info.size as usize)?;

                e.insert(str.clone());

                Ok(str)
            }
            Entry::Occupied(e) => Ok(e.get().clone()),
        }
    }
}
