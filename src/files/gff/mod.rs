use crate::error::Error::{self, *};
use crate::error::IntoParseError;

use crate::int_enum;
use rust_utils::{byte_readers::FromBytes, collect_vec::CollectVecResult};
use std::io::{Read, Seek, SeekFrom};

pub mod field;
pub mod label;
pub mod r#struct;
use field::Field;
use label::Label;
use r#struct::Struct;

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
#[repr(transparent)]
pub struct Offset(pub i32);
impl Offset {
    fn seek_to<T>(&self, read: &mut T) -> Result<u64, Error>
    where
        T: Seek,
    {
        read.seek(SeekFrom::Start(self.0 as u64)).into_parse_error()
    }
}

#[derive(Debug)]
pub struct Header {
    /// 4-char file type string
    pub file_type: String,
    /// 4-char GFF Version
    pub file_version: String,

    /// Offset of Struct array as bytes from the beginning of the file
    pub struct_offset: Offset,
    /// Number of elements in Struct array
    pub struct_count: i32,

    /// Offset of Field array as bytes from the beginning of the file
    pub field_offset: Offset,
    /// Number of elements in Field array
    pub field_count: i32,

    /// Offset of Label array as bytes from the beginning of the file
    pub label_offset: Offset,
    /// Number of elements in Label array
    pub label_count: i32,

    /// Offset of Field Data as bytes from the beginning of the file
    pub field_data_offset: Offset,
    /// Number of bytes in Field Data block
    pub field_data_count: i32,

    /// Offset of Field Indices array as bytes from the beginning of the file
    pub field_indices_offset: Offset,
    /// Number of bytes in Field Indices array
    pub field_indices_count: i32,

    /// Offset of List Indices array as bytes from the beginning of the file
    pub list_indices_offset: Offset,
    /// Number of bytes in List Indices array
    pub list_indices_count: i32,
}
impl Header {
    fn read<T: Read>(data: &mut T) -> Result<Self, Error> {
        let from_bytes = |data: &mut T| i32::from_bytes_le(data).into_parse_error();

        Ok(Self {
            file_type: read_string(data, 4)?,
            file_version: read_string(data, 4)?,

            struct_offset: Offset(from_bytes(data)?),
            struct_count: from_bytes(data)?,

            field_offset: Offset(from_bytes(data)?),
            field_count: from_bytes(data)?,

            label_offset: Offset(from_bytes(data)?),
            label_count: from_bytes(data)?,

            field_data_offset: Offset(from_bytes(data)?),
            field_data_count: from_bytes(data)?,

            field_indices_offset: Offset(from_bytes(data)?),
            field_indices_count: from_bytes(data)?,

            list_indices_offset: Offset(from_bytes(data)?),
            list_indices_count: from_bytes(data)?,
        })
    }
}

#[derive(Debug)]
pub struct Gff {
    pub header: Header,
    pub structs: Vec<Struct>,
    pub fields: Vec<Field>,
    pub labels: Vec<Label>,
    pub field_data: Vec<u8>,
    pub field_indices: Vec<i32>,
    pub list_indices: Vec<i32>,
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

        let field_indices = {
            const INDEX_SIZE: i32 = size_of::<i32>() as i32;
            (0..header.field_indices_count / INDEX_SIZE)
                .map(|_| i32::from_bytes_le(&mut data))
                .collect_vec_result()
                .into_parse_error()
        }?;

        header.list_indices_offset.seek_to(&mut data)?;

        let list_indices = {
            const INDEX_SIZE: i32 = size_of::<i32>() as i32;
            (0..header.list_indices_count / INDEX_SIZE)
                .map(|_| i32::from_bytes_le(&mut data))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn x() {
        let file = Cursor::new(include_bytes!("../../tests/files/playerlist.ifo"));
        let file = Gff::read(file).unwrap();

        let top_level = &file.structs[0];
        let field = top_level.get_field(&file.fields, 0).unwrap();
        println!("{:?}", field);

        println!("{:?}", field.get_label(&file.labels).get_string());

        panic!()
    }
}
