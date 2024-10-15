use crate::error::Error;
use super::{
    exo_string::{ExoLocString, ExoString},
    label::Label,
    r#struct::Struct,
    bin::FieldType,
};
use crate::
    files::{gff::void::Void, res_ref::ResRef}
;

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

macro_rules! expect_field {
    ($fn_name: ident, $variant: ident, $ret: ty) => {
        pub fn $fn_name(self) -> Result<$ret, Error> {
            use Field::*;
            match self {
                $variant(x) => Ok(x),
                x => Err(Error::EnumError {
                    enum_type: "Field",
                    msg: format!("Expected {} but found {:?}", stringify!($variant), x),
                }),
            }
        }
    };
}

impl Field {
    pub fn get_field_index(&self) -> FieldType {
        match self {
            Field::Byte(_) => FieldType::Byte,
            Field::CExoLocString(_) => FieldType::CExoLocString,
            Field::CExoString(_) => FieldType::CExoString,
            Field::Char(_) => FieldType::Char,
            Field::CResRef(_) => FieldType::ResRef,
            Field::Double(_) => FieldType::Double,
            Field::DWord(_) => FieldType::DWord,
            Field::DWord64(_) => FieldType::DWord64,
            Field::Float(_) => FieldType::Float,
            Field::Int(_) => FieldType::Int,
            Field::Int64(_) => FieldType::Int64,
            Field::Short(_) => FieldType::Short,
            Field::Void(_) => FieldType::Void,
            Field::Word(_) => FieldType::Word,
            Field::Struct(_) => FieldType::Struct,
            Field::List(_) => FieldType::List,
        }
    }

    expect_field!(expect_dword, DWord, u32);
    expect_field!(expect_float, Float, f32);



}

#[derive(PartialEq)]
pub struct LabeledField {
    pub label: Label,
    pub field: Field,
}
impl LabeledField {
    pub fn new(label: Label, field: Field) -> Self {
        Self { label, field }
    }
}
impl std::fmt::Debug for LabeledField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "\"{}\": {:#?}", &self.label.as_str(), &self.field)
        } else {
            write!(f, "\"{}\": {:?}", &self.label.as_str(), &self.field)
        }
    }
}
