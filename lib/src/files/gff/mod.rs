// Note to self: type names ending Data usually means data as read from the file,
// i.e. before being resolved into something more useable

use super::{from_bytes_le, read_string, Offset};
use crate::error::{Error, IntoError};

use rust_utils::{byte_readers::FromBytes, collect_vec::CollectVecResult};
use std::io::{Read, Seek};

pub mod exo_string;
pub mod field;
pub mod label;
pub mod r#struct;
pub mod void;
use field::FieldData;
use label::Label;
use r#struct::StructData;

const INDEX_SIZE: i32 = 4;

#[derive(Debug, Default)]
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
    fn read(mut data: impl Read) -> Result<Self, Error> {
        Ok(Self {
            file_type: read_string(&mut data, 4)?,
            file_version: read_string(&mut data, 4)?,

            struct_offset: Offset(from_bytes_le(&mut data)?),
            struct_count: from_bytes_le(&mut data)?,

            field_offset: Offset(from_bytes_le(&mut data)?),
            field_count: from_bytes_le(&mut data)?,

            label_offset: Offset(from_bytes_le(&mut data)?),
            label_count: from_bytes_le(&mut data)?,

            field_data_offset: Offset(from_bytes_le(&mut data)?),
            field_data_count: from_bytes_le(&mut data)?,

            field_indices_offset: Offset(from_bytes_le(&mut data)?),
            field_indices_count: from_bytes_le(&mut data)?,

            list_indices_offset: Offset(from_bytes_le(&mut data)?),
            list_indices_count: from_bytes_le(&mut data)?,
        })
    }
}

#[derive(Debug, Default)]
pub struct GffData {
    pub header: Header,
    pub structs: Vec<StructData>,
    pub fields: Vec<FieldData>,
    pub labels: Vec<Label>,
    pub field_data: Vec<u8>,
    pub field_indices: Vec<i32>,
    pub list_indices: Vec<i32>,
}
impl GffData {
    pub fn read(mut data: impl Read + Seek) -> Result<Self, Error> {
        let header = Header::read(&mut data)?;

        header.struct_offset.seek_to(&mut data)?;

        let structs = (0..header.struct_count)
            .map(|_| StructData::read(&mut data))
            .collect_vec_result()?;

        header.field_offset.seek_to(&mut data)?;

        let fields = (0..header.field_count)
            .map(|_| FieldData::read(&mut data))
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
    use crate::files::tlk::Tlk;

    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_test() {
        let file = Cursor::new(include_bytes!("../../tests/files/playerlist.ifo"));
        let file = GffData::read(file).unwrap();

        let tlk = Tlk::read(Cursor::new(include_bytes!("../../tests/files/dialog.TLK"))).unwrap();

        let structs = file.structs
            .iter()
            .map(|s| {
                s.resolve(&file, &tlk)
            })
        .collect_vec_result();

        for (i, s) in structs.into_iter().enumerate() {
            println!("{i}: {s:#?}");
        }
    }
}
