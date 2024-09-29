pub mod gff;
mod res_ref;
pub mod tlk;
pub mod two_da;

use crate::{
    error::{Error, IntoError},
    int_enum,
};
use rust_utils::byte_readers::FromBytes;
use std::io::{Read, Seek, SeekFrom};

int_enum! { Language,
    English, 0,
    French, 1,
    German, 2,
    Italian, 3,
    Spanish, 4,
    Polish, 5,
    Korean, 128,
    ChineseTraditional, 129,
    ChineseSimplified, 130,
    Japanese, 131
}
impl Default for Language {
    fn default() -> Self {
        Self::English
    }
}

int_enum! { Gender,
    Masculine, 0,
    Feminine, 1
}

impl Default for Gender {
    fn default() -> Self {
        Self::Masculine
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Offset(pub i32);
impl Offset {
    pub fn seek_to<T>(&self, read: &mut T) -> Result<u64, Error>
    where
        T: Seek,
    {
        read.seek(SeekFrom::Start(self.0 as u64)).into_parse_error()
    }

    pub fn seek_with_offet<T: Seek>(&self, read: &mut T, offset: u64) -> Result<u64, Error> {
        let pos = (self.0 as u64) + offset;
        read.seek(SeekFrom::Start(pos)).into_parse_error()
    }
}

fn read_string<R: Read>(data: &mut R, len: usize) -> Result<String, Error> {
    let mut strbuf = vec![0u8; len];

    let to_str = |v: &[u8]| String::from_utf8_lossy(v).to_string();

    data.read_exact(strbuf.as_mut())
        .into_parse_error()
        .map(|_| to_str(&strbuf))
}

fn read_bytes<const N: usize, R: Read>(data: &mut R) -> Result<[u8; N], Error> {
    let mut buf = [0u8; N];
    data.read_exact(&mut buf).into_parse_error()?;
    Ok(buf)
}

fn from_bytes_le<T>(data: impl Read) -> Result<T, Error>
where
    T: FromBytes,
    T::Error: std::error::Error,
{
    T::from_bytes_le(data).into_parse_error()
}
