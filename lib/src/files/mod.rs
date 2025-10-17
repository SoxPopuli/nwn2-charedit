pub mod gff;
pub mod offset;
pub mod res_ref;
pub mod tlk;
pub mod two_da;

use crate::error::{Error, IntoError};
use common::int_enum;
pub use offset::Offset;
use rust_utils::byte_readers::FromBytes;
use std::io::{Read, Write};

int_enum! {
    pub enum Language: u8 {
        English = 0,
        French = 1,
        German = 2,
        Italian = 3,
        Spanish = 4,
        Polish = 5,
        Korean = 128,
        ChineseTraditional = 129,
        ChineseSimplified = 130,
        Japanese = 131,
    }
}
impl Default for Language {
    fn default() -> Self {
        Self::English
    }
}

int_enum! {
    pub enum Gender: u8 {
        Masculine = 0,
        Feminine = 1
    }
}

impl Default for Gender {
    fn default() -> Self {
        Self::Masculine
    }
}

fn read_string<R: Read>(data: &mut R, len: usize) -> Result<String, Error> {
    let mut strbuf = vec![0u8; len];

    let to_str = |v: &[u8]| String::from_utf8_lossy(v).to_string();

    data.read_exact(strbuf.as_mut())
        .into_parse_error()
        .map(|_| to_str(&strbuf))
}

fn from_bytes_le<T>(data: impl Read) -> Result<T, Error>
where
    T: FromBytes,
    T::Error: std::error::Error,
{
    T::from_bytes_le(data).into_parse_error()
}

fn write_all<W: Write>(writer: &mut W, data: &[u8]) -> Result<(), Error> {
    writer.write_all(data).into_write_error()
}
