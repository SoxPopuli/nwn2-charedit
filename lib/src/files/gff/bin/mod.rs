use super::{label::Label, Header};
use crate::{
    error::{Error, IntoError},
    files::{
        from_bytes_le,
        gff::{
            exo_string::{ExoLocString, ExoString},
            void::Void,
        },
        res_ref::ResRef,
        tlk::Tlk,
    },
    int_enum,
};
use rust_utils::collect_vec::CollectVecResult;
use std::io::{Read, Seek};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Gff {
    pub header: Header,
    pub structs: Vec<Struct>,
    pub fields: Vec<Field>,
    pub labels: Vec<Label>,
    pub field_data: Vec<u8>,
    pub field_indices: Vec<u32>,
    pub list_indices: Vec<u32>,
}
impl Gff {
    pub fn read(mut data: impl Read + Seek) -> Result<Self, Error> {
        let header = Header::read(&mut data)?;

        header.struct_offset.seek_to(&mut data)?;

        let structs = (0..header.struct_count)
            .map(|_| Struct::read(&mut data))
            .collect_vec_result()?;

        header.field_offset.seek_to(&mut data)?;

        let fields = (0..header.field_count)
            .map(|_| Field::read(&mut data))
            .collect_vec_result()?;

        header.label_offset.seek_to(&mut data)?;

        let labels = (0..header.label_count)
            .map(|_| Label::read(&mut data))
            .collect_vec_result()?;

        header.field_data_offset.seek_to(&mut data)?;

        let field_data = {
            let mut buf = vec![0u8; header.field_data_count as usize];
            data.read_exact(&mut buf).into_parse_error()?;
            buf
        };

        header.field_indices_offset.seek_to(&mut data)?;
        const INDEX_SIZE: u32 = size_of::<u32>() as u32;

        let field_indices = {
            (0..header.field_indices_count / INDEX_SIZE)
                .map(|_| from_bytes_le(&mut data))
                .collect_vec_result()
                .into_parse_error()
        }?;

        header.list_indices_offset.seek_to(&mut data)?;

        let list_indices = {
            (0..header.list_indices_count / INDEX_SIZE)
                .map(|_| from_bytes_le(&mut data))
                .collect_vec_result()
                .into_parse_error()
        }?;

        Ok(Self {
            header,
            structs,
            fields,
            labels,
            field_data,
            field_indices,
            list_indices,
        })
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Struct {
    pub id: u32,
    pub data_or_data_offset: u32,
    pub field_count: u32,
}
impl Struct {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let struct_type = from_bytes_le(&mut data)?;
        let data_or_data_offset = from_bytes_le(&mut data)?;
        let field_count = from_bytes_le(&mut data)?;

        Ok(Self {
            id: struct_type,
            data_or_data_offset,
            field_count,
        })
    }

    pub fn get_field<'a>(&self, file: &'a Gff, index: u32) -> Option<&'a Field> {
        if index >= self.field_count {
            return None;
        }

        if self.field_count == 1 {
            // Index into field array
            let field = &file.fields[self.data_or_data_offset as usize];
            Some(field)
        } else {
            // Byte offset into field indices
            assert!(
                self.data_or_data_offset % 4 == 0,
                "Data index {} not aligned on u32 boundary :(",
                self.data_or_data_offset
            );

            let index = (self.data_or_data_offset / 4) + index;
            let field_index = file.field_indices[index as usize];
            let field = &file.fields[field_index as usize];

            Some(field)
        }
    }
}

fn shrink_array<const BIG: usize, const SMALL: usize>(x: &[u8; BIG]) -> [u8; SMALL] {
    assert!(BIG >= SMALL, "Tried to shrink {x:?} to size {SMALL}");

    std::array::from_fn(|i| x[i])
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field {
    pub id: FieldType,
    pub label_index: u32,
    pub data_or_data_offset: u32,
}
impl Field {
    fn read(mut data: impl Read) -> Result<Self, Error> {
        let index = {
            let index: u32 = from_bytes_le(&mut data)?;
            FieldType::try_from(index as u8)?
        };
        let label_index = from_bytes_le(&mut data)?;
        let data_or_data_offset = from_bytes_le(&mut data)?;

        Ok(Self {
            id: index,
            label_index,
            data_or_data_offset,
        })
    }

    pub fn to_field<R>(&self, file: &Gff, tlk: &Tlk<R>) -> Result<super::field::Field, Error>
    where
        R: Read + Seek,
    {
        const INDEX_SIZE: u32 = size_of::<u32>() as u32;

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

        use super::field::Field;

        fn field_data_offset(file: &Gff, offset: u32) -> &[u8] {
            &file.field_data[offset as usize..]
        }

        match self.id {
            FieldType::Byte => {
                let bytes = self.data_or_data_offset.to_le_bytes();
                Ok(Field::Byte(bytes[0]))
            }
            FieldType::Char => {
                let bytes = self.data_or_data_offset.to_le_bytes();
                let char = bytes[0] as char;
                Ok(Field::Char(char))
            }
            FieldType::Word => Ok(Field::Word(read_smaller!(u16))),
            FieldType::Short => Ok(Field::Short(read_smaller!(i16))),
            FieldType::DWord => Ok(Field::DWord(self.data_or_data_offset)),
            FieldType::Int => Ok(Field::Int(self.data_or_data_offset as i32)),
            FieldType::DWord64 => Ok(Field::DWord64(read_complex!(u64, file.field_data))),
            FieldType::Int64 => Ok(Field::Int64(read_complex!(i64, file.field_data))),
            FieldType::Float => Ok(Field::Float(read_smaller!(f32))),
            FieldType::Double => Ok(Field::Double(read_complex!(f64, file.field_data))),
            FieldType::CExoString => {
                let mut data = field_data_offset(file, self.data_or_data_offset);

                let exo_string = ExoString::read(&mut data)?;
                Ok(Field::CExoString(exo_string))
            }
            FieldType::ResRef => {
                let mut data = field_data_offset(file, self.data_or_data_offset);

                let res_ref = ResRef::read(&mut data)?;
                Ok(Field::CResRef(res_ref))
            }
            FieldType::CExoLocString => {
                let mut data = field_data_offset(file, self.data_or_data_offset);

                let s = ExoLocString::read(&mut data, tlk)?;

                Ok(Field::CExoLocString(s))
            }
            FieldType::Void => {
                let mut data = field_data_offset(file, self.data_or_data_offset);

                Ok(Field::Void(Void::read(&mut data)?))
            }
            FieldType::Struct => {
                let index = self.data_or_data_offset as usize;
                let s = &file.structs[index];

                Ok(Field::Struct(super::Struct::new(s, file, tlk)?))
            }
            FieldType::List => {
                let index = (self.data_or_data_offset / INDEX_SIZE) as usize;
                let struct_count = file.list_indices[index] as usize;

                let start = index + 1;
                let end = start + struct_count;

                let structs = file.list_indices[start..end]
                    .iter()
                    .map(|i| {
                        let s = &file.structs[*i as usize];
                        super::Struct::new(s, file, tlk)
                    })
                    .collect_vec_result()?;

                Ok(Field::List(structs))
            }
        }
    }
}

int_enum! { FieldType,
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
impl FieldType {
    // A type is complex if it can't be represented using only 4 bytes
    pub fn is_complex(&self) -> bool {
        match self {
            FieldType::Byte
            | FieldType::Char
            | FieldType::Word
            | FieldType::Short
            | FieldType::DWord
            | FieldType::Int
            | FieldType::Float => false,
            FieldType::DWord64
            | FieldType::Int64
            | FieldType::Double
            | FieldType::CExoString
            | FieldType::ResRef
            | FieldType::CExoLocString
            | FieldType::Void
            | FieldType::Struct
            | FieldType::List => true,
        }
    }
}
