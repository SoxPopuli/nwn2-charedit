use crate::error::Error::{self, *};
use rust_utils::byte_readers::FromBytes;
use std::io::{Read, Seek};

macro_rules! int_enum {
    ($name: ident, $($case: ident, $val: expr),+) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name {
            $($case = $val),+
        }

        impl TryFrom<u8> for $name {
            type Error = crate::error::Error;
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                use $name::*;
                match value {
                    $($val => Ok($case)),+,
                    _ => Err(EnumError{
                        enum_type: stringify!($name),
                        msg: format!("Unexpected value: {value}")
                    }),
                }
            }
        }

        impl From<$name> for u8 {
            fn from(value: $name) -> u8 {
                use $name::*;
                match value {
                    $($case => $val),+
                }
            }
        }
    };
}

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

fn read_string<T: Read>(data: &mut T, len: usize) -> Result<String, Error> {
    let mut strbuf = vec![0u8; len];

    let to_str = |v| String::from_utf8(v).map_err(|e| ParseError(e.to_string()));

    data.read_exact(strbuf.as_mut())
        .map_err(|e| ParseError(e.to_string()))
        .and_then(|_| to_str(strbuf))
}

#[derive(Debug)]
pub struct Header {
    /// 4-char file type string
    pub file_type: String,
    /// 4-char GFF Version
    pub file_version: String,

    /// Offset of Struct array as bytes from the beginning of the file
    pub struct_offset: i32,
    /// Number of elements in Struct array
    pub struct_count: i32,

    /// Offset of Field array as bytes from the beginning of the file
    pub field_offset: i32,
    /// Number of elements in Field array
    pub field_count: i32,

    /// Offset of Label array as bytes from the beginning of the file
    pub label_offset: i32,
    /// Number of elements in Label array
    pub label_count: i32,

    /// Offset of Field Data as bytes from the beginning of the file
    pub field_data_offset: i32,
    /// Number of bytes in Field Data block
    pub field_data_count: i32,

    /// Offset of Field Indices array as bytes from the beginning of the file
    pub field_indices_offset: i32,
    /// Number of bytes in Field Indices array
    pub field_indices_count: i32,

    /// Offset of List Indices array as bytes from the beginning of the file
    pub list_indices_offset: i32,
    /// Number of bytes in List Indices array
    pub list_indices_count: i32,
}
impl Header {
    fn read<T: Read>(data: &mut T) -> Result<Self, Error> {
        let from_bytes =
            |data: &mut T| i32::from_bytes_le(data).map_err(|e| ParseError(e.to_string()));

        Ok(Self {
            file_type: read_string(data, 4)?,
            file_version: read_string(data, 4)?,

            struct_offset: from_bytes(data)?,
            struct_count: from_bytes(data)?,

            field_offset: from_bytes(data)?,
            field_count: from_bytes(data)?,

            label_offset: from_bytes(data)?,
            label_count: from_bytes(data)?,

            field_data_offset: from_bytes(data)?,
            field_data_count: from_bytes(data)?,

            field_indices_offset: from_bytes(data)?,
            field_indices_count: from_bytes(data)?,

            list_indices_offset: from_bytes(data)?,
            list_indices_count: from_bytes(data)?,
        })
    }
}

int_enum!{ FieldType,
    Byte, 0
}

#[derive(Debug)]
pub struct Struct {
    pub struct_type: i32,
    // pub data: StructData,
}

fn parse(mut data: impl Read + Seek) {
    let header = Header::read(&mut data).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn x() {
        let file = Cursor::new(include_bytes!("../tests/files/playerlist.ifo"));
        parse(file);
    }
}
