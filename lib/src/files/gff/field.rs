use std::io::{Read, Seek};

use rust_utils::{byte_readers::from_bytes_le, collect_vec::CollectVecResult, pipe::Pipe};

use super::{
    exo_string::{ExoLocString, ExoString},
    label::Label,
    r#struct::Struct,
    GffData, INDEX_SIZE,
};
use crate::{
    error::{Error, IntoError},
    files::{gff::void::Void, res_ref::ResRef, tlk::Tlk},
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

fn shrink_array<const BIG: usize, const SMALL: usize>(x: &[u8; BIG]) -> [u8; SMALL] {
    assert!(BIG <= SMALL, "Target array is not smaller than source");

    std::array::from_fn(|i| x[i])
}

#[derive(Debug)]
pub struct FieldData {
    pub field_type: FieldIndex,
    pub label_index: i32,
    pub data_or_data_offset: i32,
}
impl FieldData {
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

    fn get_field_data_offset<'a>(&'a self, file: &'a GffData) -> &'a [u8] {
        let offset = self.data_or_data_offset as usize;
        &file.field_data[offset..]
    }

    pub fn get_data<R>(&self, file: &GffData, tlk: &Tlk<R>) -> Result<Field, Error>
    where
        R: Read + Seek,
    {
        macro_rules! read_smaller {
            ($t: ty) => {{
                let bytes = self.data_or_data_offset.to_le_bytes();
                let data = <$t>::from_le_bytes(shrink_array(&bytes));

                data
            }};
        }

        macro_rules! read_complex {
            ($t: ty, $data_source: expr) => {{
                const DATA_SIZE: usize = size_of::<$t>();

                let index = self.data_or_data_offset as usize;
                let data = &$data_source[index..index + DATA_SIZE];

                let mut buf = [0u8; DATA_SIZE];
                buf.copy_from_slice(data);

                <$t>::from_le_bytes(buf)
            }};
        }

        match self.field_type {
            FieldIndex::Byte => {
                let bytes = self.data_or_data_offset.to_le_bytes();
                Ok(Field::Byte(bytes[0]))
            }
            FieldIndex::Char => {
                let bytes = self.data_or_data_offset.to_le_bytes();
                let char = bytes[0] as char;
                Ok(Field::Char(char))
            }
            FieldIndex::Word => Ok(Field::Word(read_smaller!(u16))),
            FieldIndex::Short => Ok(Field::Short(read_smaller!(i16))),
            FieldIndex::DWord => Ok(Field::DWord(self.data_or_data_offset as u32)),
            FieldIndex::Int => Ok(Field::Int(self.data_or_data_offset)),
            FieldIndex::DWord64 => Ok(Field::DWord64(read_complex!(u64, file.field_data))),
            FieldIndex::Int64 => Ok(Field::Int64(read_complex!(i64, file.field_data))),
            FieldIndex::Float => Ok(Field::Float(read_smaller!(f32))),
            FieldIndex::Double => Ok(Field::Double(read_complex!(f64, file.field_data))),
            FieldIndex::CExoString => {
                let mut data = self.get_field_data_offset(file);

                ExoString::read(&mut data)?.pipe(Field::CExoString).pipe(Ok)
            }
            FieldIndex::ResRef => {
                let mut data = self.get_field_data_offset(file);

                let res_ref = ResRef::read(&mut data)?;
                Ok(Field::CResRef(res_ref))
            }
            FieldIndex::CExoLocString => {
                let mut data = self.get_field_data_offset(file);

                let s = ExoLocString::read(&mut data, tlk)?;

                Ok(Field::CExoLocString(s))
            }
            FieldIndex::Void => {
                let mut data = self.get_field_data_offset(file);

                Ok(Field::Void(Void::read(&mut data)?))
            }
            FieldIndex::Struct => {
                let index = self.data_or_data_offset as usize;
                let s = file.structs[index].resolve(file, tlk)?;

                Ok(Field::Struct(s))
            }
            FieldIndex::List => {
                let index = (self.data_or_data_offset / INDEX_SIZE) as usize;
                let struct_count = file.list_indices[index] as usize;

                let start = index + 1;
                let end = start + struct_count;

                let structs = file.list_indices[start..end]
                    .iter()
                    .map(|i| file.structs[*i as usize].resolve(file, tlk))
                    .collect_vec_result()?;

                Ok(Field::List(structs))
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

#[derive(Debug, PartialEq)]
pub enum Field {
    Byte(u8),
    CExoLocString(ExoLocString),
    CExoString(ExoString),
    Char(char),
    CResRef(ResRef),
    Double(f64),
    DWord(u32),
    DWord64(u64),
    Float(f32),
    Int(i32),
    Int64(i64),
    Short(i16),
    Void(Void),
    Word(u16),
    Struct(Struct),
    List(Vec<Struct>),
}

impl Field {
    pub fn get_field_index(&self) -> FieldIndex {
        match self {
            Field::Byte(_) => FieldIndex::Byte,
            Field::CExoLocString(_) => FieldIndex::CExoLocString,
            Field::CExoString(_) => FieldIndex::CExoString,
            Field::Char(_) => FieldIndex::Char,
            Field::CResRef(_) => FieldIndex::ResRef,
            Field::Double(_) => FieldIndex::Double,
            Field::DWord(_) => FieldIndex::DWord,
            Field::DWord64(_) => FieldIndex::DWord64,
            Field::Float(_) => FieldIndex::Float,
            Field::Int(_) => FieldIndex::Int,
            Field::Int64(_) => FieldIndex::Int64,
            Field::Short(_) => FieldIndex::Short,
            Field::Void(_) => FieldIndex::Void,
            Field::Word(_) => FieldIndex::Word,
            Field::Struct(_) => FieldIndex::Struct,
            Field::List(_) => FieldIndex::List,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LabeledField {
    pub label: Label,
    pub field: Field,
}

impl LabeledField {
    pub fn new(label: Label, field: Field) -> Self {
        Self { label, field }
    }
}
