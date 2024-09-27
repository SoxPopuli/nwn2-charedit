use std::io::Read;

use rust_utils::{byte_readers::from_bytes_le, collect_vec::CollectVec};

use super::{label::Label, r#struct::Struct, Gff};
use crate::{
    error::{Error, IntoParseError},
    int_enum,
};

// | Field Type    | Size (in bytes) | Description                                                                                                                                                                                                            |
// | ------------- | --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
// | BYTE          | 1               | Unsigned single byte (0 to 255)                                                                                                                                                                                        |
// | CExoLocString | variable        | Localized string. Contains a StringRef DWORD, and a number of CExoStrings, each having their own language ID.                                                                                                          |
// | CExoString    | variable        | Non-localized string                                                                                                                                                                                                   |
// | CHAR          | 1               | Single character byte                                                                                                                                                                                                  |
// | CResRef       | 16              | Filename of a game resource. Max length is 16 characters. Unused characters are nulls.                                                                                                                                 |
// | DOUBLE        | 8               | Double-precision floating point value                                                                                                                                                                                  |
// | DWORD         | 4               | Unsigned integer (0 to 4294967296)                                                                                                                                                                                     |
// | DWORD64       | 8               | Unsigned integer (0 to roughly 18E18)                                                                                                                                                                                  |
// | FLOAT         | 4               | Floating point value                                                                                                                                                                                                   |
// | INT           | 4               | Signed integer (-2147483648 to 2147483647)                                                                                                                                                                             |
// | INT64         | 8               | Signed integer (roughly -9E18 to +9E18)                                                                                                                                                                                |
// | SHORT         | 2               | Signed integer (-32768 to 32767)                                                                                                                                                                                       |
// | VOID          | variable        | Variable-length arbitrary data                                                                                                                                                                                         |
// | WORD          | 2               | Unsigned integer value (0 to 65535)                                                                                                                                                                                    |
// | Struct        | variable        | A complex data type that can contain any number of any of the other data types, including other Structs.                                                                                                               |
// | List          | variable        | A list of Structs                                                                                                                                                                                                      |

// | Type ID | Type          | Complex? |
// | ------- | ------------- | -------- |
// | 0       | BYTE          |          |
// | 1       | CHAR          |          |
// | 2       | WORD          |          |
// | 3       | SHORT         |          |
// | 4       | DWORD         |          |
// | 5       | INT           |          |
// | 6       | DWORD64       | yes      |
// | 7       | INT64         | yes      |
// | 8       | FLOAT         |          |
// | 9       | DOUBLE        | yes      |
// | 10      | CExoString    | yes      |
// | 11      | ResRef        | yes      |
// | 12      | CExoLocString | yes      |
// | 13      | VOID          | yes      |
// | 14      | Struct        | yes*     |
// | 15      | List          | yes**    |

#[derive(Debug)]
pub struct Field {
    pub field_type: FieldIndex,
    pub label_index: i32,
    pub data_or_data_offset: i32,
}
impl Field {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let field_type = from_bytes_le::<i32>(&mut data)
            .into_parse_error()
            .and_then(|x| (x as u8).try_into())?;

        Ok(Self {
            field_type,
            label_index: from_bytes_le(&mut data).into_parse_error()?,
            data_or_data_offset: from_bytes_le(&mut data).into_parse_error()?,
        })
    }

    pub fn get_label<'a>(&self, labels: &'a [Label]) -> &'a Label {
        &labels[self.label_index as usize]
    }

    pub fn get_data<'a>(
        &self,
        structs: &'a [Struct],
        field_data: &'a [u8],
        list_indices: &'a [i32],
    ) -> FieldData {
        match self.field_type {
            FieldIndex::Byte => todo!(),
            FieldIndex::Char => todo!(),
            FieldIndex::Word => todo!(),
            FieldIndex::Short => todo!(),
            FieldIndex::DWord => todo!(),
            FieldIndex::Int => todo!(),
            FieldIndex::DWord64 => todo!(),
            FieldIndex::Int64 => todo!(),
            FieldIndex::Float => todo!(),
            FieldIndex::Double => todo!(),
            FieldIndex::CExoString => todo!(),
            FieldIndex::ResRef => todo!(),
            FieldIndex::CExoLocString => todo!(),
            FieldIndex::Void => todo!(),
            FieldIndex::Struct => todo!(),
            FieldIndex::List => {
                let index = (self.data_or_data_offset / 4) as usize;
                let struct_count = list_indices[index] as usize;

                let structs = list_indices[index + 1..struct_count]
                    .iter()
                    .map(|i| structs[*i as usize].clone())
                    .collect_vec();

                FieldData::List(structs)
            }
        }
    }
}

int_enum! { FieldIndex,
    Byte, 0,
    Char, 1,
    Word, 2,
    Short, 3,
    DWord, 4,
    Int, 5,
    DWord64, 6,
    Int64, 7,
    Float, 8,
    Double, 9,
    CExoString, 10,
    ResRef, 11,
    CExoLocString, 12,
    Void, 13,
    Struct, 14,
    List, 15
}
impl FieldIndex {
    // A type is complex if it can't be represented using only 4 bytes
    pub fn is_complex(&self) -> bool {
        match self {
            FieldIndex::Byte
            | FieldIndex::Char
            | FieldIndex::Word
            | FieldIndex::Short
            | FieldIndex::DWord
            | FieldIndex::Int
            | FieldIndex::Float => false,
            FieldIndex::DWord64
            | FieldIndex::Int64
            | FieldIndex::Double
            | FieldIndex::CExoString
            | FieldIndex::ResRef
            | FieldIndex::CExoLocString
            | FieldIndex::Void
            | FieldIndex::Struct
            | FieldIndex::List => true,
        }
    }
}

#[derive(Debug)]
pub enum FieldData {
    Byte(u8),
    CExoLocString(String),
    CExoString(String),
    Char(char),
    CResRef(String),
    Double(f64),
    DWord(u32),
    DWord64(u64),
    Float(f32),
    Int(i32),
    Int64(i64),
    Short(i16),
    Void(Vec<u8>),
    Word(u16),
    Struct(Struct),
    List(Vec<Struct>),
}

impl FieldData {
    pub fn get_field_index(&self) -> FieldIndex {
        match self {
            FieldData::Byte(_) => FieldIndex::Byte,
            FieldData::CExoLocString(_) => FieldIndex::CExoLocString,
            FieldData::CExoString(_) => FieldIndex::CExoString,
            FieldData::Char(_) => FieldIndex::Char,
            FieldData::CResRef(_) => FieldIndex::ResRef,
            FieldData::Double(_) => FieldIndex::Double,
            FieldData::DWord(_) => FieldIndex::DWord,
            FieldData::DWord64(_) => FieldIndex::DWord64,
            FieldData::Float(_) => FieldIndex::Float,
            FieldData::Int(_) => FieldIndex::Int,
            FieldData::Int64(_) => FieldIndex::Int64,
            FieldData::Short(_) => FieldIndex::Short,
            FieldData::Void(_) => FieldIndex::Void,
            FieldData::Word(_) => FieldIndex::Word,
            FieldData::Struct(_) => FieldIndex::Struct,
            FieldData::List(_) => FieldIndex::List,
        }
    }
}
