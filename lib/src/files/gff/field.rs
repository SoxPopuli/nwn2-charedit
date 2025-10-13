use super::{
    bin::FieldType,
    exo_string::{ExoLocString, ExoString},
    label::Label,
    r#struct::Struct,
};
use crate::error::Error;
use crate::files::{gff::void::Void, res_ref::ResRef};
use paste::paste;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[repr(transparent)]
pub struct U32Char(pub u32);
impl U32Char {
    pub fn get_char(&self) -> Option<char> {
        encoding_rs::WINDOWS_1252
            .decode_without_bom_handling(&self.0.to_le_bytes())
            .0
            .chars()
            .next()
    }

    pub fn set_char(&mut self, c: char) {
        self.0 = c as u32;
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Field {
    Byte(u8),
    ExoLocString(ExoLocString),
    ExoString(ExoString),
    /// Sometimes char values can be above 255 (e.g. `u32::MAX`),
    /// therefore we store the entire u32 value
    Char(U32Char),
    ResRef(ResRef),
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

macro_rules! impl_expect_field {
    (ref $variant: ident, $ret: ty) => {
        paste! {
            pub fn [< expect_ $variant:lower >](&self) -> Result<&$ret, Error> {
                use Field::*;
                match self {
                    $variant(x) => Ok(x),
                    x => Err(Error::EnumError {
                        enum_type: "Field",
                        msg: format!("Expected {} but found {:?}", stringify!($variant), x),
                    }),
                }
            }

            pub fn [< try_ $variant:lower >](&self) -> Option<&$ret> {
                use Field::*;
                match self {
                    $variant(x) => Some(x),
                    _ => None,
                }
            }
        }
    };
    ($variant: ident, $ret: ty) => {
        paste! {
            pub fn [< expect_ $variant:lower >](&self) -> Result<$ret, Error> {
                use Field::*;
                match self {
                    $variant(x) => Ok(*x),
                    x => Err(Error::EnumError {
                        enum_type: "Field",
                        msg: format!("Expected {} but found {:?}", stringify!($variant), x),
                    }),
                }
            }

            pub fn [< try_ $variant:lower >](&self) -> Option<$ret> {
                use Field::*;
                match self {
                    $variant(x) => Some(*x),
                    _ => None,
                }
            }
        }
    };
}

macro_rules! impl_into_field {
    ($t: ty, $variant: ident) => {
        impl From<$t> for Field {
            fn from(value: $t) -> Field {
                use Field::*;
                $variant(value)
            }
        }
    };
}

impl_into_field!(u8, Byte);
impl_into_field!(ExoLocString, ExoLocString);
impl_into_field!(ExoString, ExoString);
impl_into_field!(U32Char, Char);
impl_into_field!(ResRef, ResRef);
impl_into_field!(f64, Double);
impl_into_field!(u32, DWord);
impl_into_field!(u64, DWord64);
impl_into_field!(f32, Float);
impl_into_field!(i32, Int);
impl_into_field!(i64, Int64);
impl_into_field!(i16, Short);
impl_into_field!(Void, Void);
impl_into_field!(u16, Word);
impl_into_field!(Struct, Struct);
impl_into_field!(Vec<Struct>, List);

impl Field {
    pub fn get_field_type(&self) -> FieldType {
        match self {
            Field::Byte(_) => FieldType::Byte,
            Field::ExoLocString(_) => FieldType::ExoLocString,
            Field::ExoString(_) => FieldType::ExoString,
            Field::Char(_) => FieldType::Char,
            Field::ResRef(_) => FieldType::ResRef,
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

    impl_expect_field!(Byte, u8);
    impl_expect_field!(Char, U32Char);
    impl_expect_field!(Double, f64);
    impl_expect_field!(DWord64, u64);
    impl_expect_field!(DWord, u32);
    impl_expect_field!(ref ExoLocString, ExoLocString);
    impl_expect_field!(ref ExoString, ExoString);
    impl_expect_field!(Float, f32);
    impl_expect_field!(Int64, i64);
    impl_expect_field!(Int, i32);
    impl_expect_field!(ref List, Vec<Struct>);
    impl_expect_field!(ref ResRef, ResRef);
    impl_expect_field!(Short, i16);
    impl_expect_field!(ref Struct, Struct);
    impl_expect_field!(ref Void, Void);
    impl_expect_field!(Word, u16);
}

#[derive(PartialEq, Clone)]
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
