use super::{Offset, from_bytes_le};
use crate::error::{Error, IntoError};
use std::{
    collections::HashMap,
    io::{Read, Seek},
    sync::{Arc, RwLock},
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

#[derive(Debug)]
pub struct TlkReaderInner<R>
where
    R: Read + Seek,
{
    pub(crate) data: R,
    pub(crate) entry_cache: HashMap<u32, Arc<str>>,
}
impl<R> Default for TlkReaderInner<R>
where
    R: Read + Seek + Default,
{
    fn default() -> Self {
        Self {
            data: R::default(),
            entry_cache: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct TlkReader<R>
where
    R: Read + Seek,
{
    pub(crate) string_info: Vec<StringInfo>,
    pub(crate) string_entry_offset: Offset,
    pub(crate) inner: RwLock<TlkReaderInner<R>>,
}
impl<R> PartialEq for TlkReader<R>
where
    R: Read + Seek,
{
    fn eq(&self, other: &Self) -> bool {
        self.string_info == other.string_info
            && self.string_entry_offset == other.string_entry_offset
    }
}

impl<R> Default for TlkReader<R>
where
    R: Read + Seek + Default,
{
    fn default() -> Self {
        Self {
            string_info: Vec::default(),
            string_entry_offset: Offset::default(),
            inner: Default::default(),
        }
    }
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
    pub fn new(string_info: Vec<StringInfo>, string_entry_offset: Offset, data: R) -> Self {
        let inner = TlkReaderInner {
            data,
            entry_cache: Default::default(),
        };

        TlkReader {
            string_info,
            string_entry_offset,
            inner: inner.into(),
        }
    }

    /// Gets str ref at index, and reads from data if not done so before
    pub(crate) fn read_index(&self, index: u32) -> Result<Arc<str>, Error> {
        let possible_entry = {
            let inner = self.inner.read().unwrap();
            inner.entry_cache.get(&index).cloned()
        };

        if let Some(entry) = possible_entry {
            Ok(entry)
        } else {
            let mut inner = self.inner.write().unwrap();

            let info = self
                .string_info
                .get(index as usize)
                .ok_or(Error::InvalidStrRef { value: index })?;

            info.offset
                .seek_with_offset(&mut inner.data, self.string_entry_offset)?;

            let str = if info.size == 0 {
                super::EMPTY_STRING.clone()
            } else {
                read_str(&mut inner.data, info.size as usize)?
            };

            inner.entry_cache.insert(index, str.clone());

            Ok(str)
        }
    }
}
