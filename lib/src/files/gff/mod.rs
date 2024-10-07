// Note to self: type names ending Data usually means data as read from the file,
// i.e. before being resolved into something more useable

use super::{from_bytes_le, read_string, Offset};
use crate::{
    error::{Error, IntoError},
    files::{tlk::Tlk, write_all},
};

use rust_utils::{byte_readers::FromBytes, collect_vec::CollectVecResult};
use std::io::{Read, Seek, Write};

pub mod exo_string;
pub mod field;
pub mod label;
pub mod r#struct;
pub mod void;
use field::{FieldData, FIELD_DATA_SIZE};
use label::{Label, LABEL_SIZE};
use r#struct::{Struct, StructData, STRUCT_DATA_SIZE};

const INDEX_SIZE: i32 = size_of::<i32>() as i32;

#[derive(Debug, Default, PartialEq, Eq)]
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

    fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        write_all(writer, &self.file_type.as_bytes()[..4])?;
        write_all(writer, &self.file_version.as_bytes()[..4])?;

        write_all(writer, &self.struct_offset.0.to_le_bytes())?;
        write_all(writer, &self.struct_count.to_le_bytes())?;

        write_all(writer, &self.field_offset.0.to_le_bytes())?;
        write_all(writer, &self.field_count.to_le_bytes())?;

        write_all(writer, &self.label_offset.0.to_le_bytes())?;
        write_all(writer, &self.label_count.to_le_bytes())?;

        write_all(writer, &self.field_data_offset.0.to_le_bytes())?;
        write_all(writer, &self.field_data_count.to_le_bytes())?;

        write_all(writer, &self.field_indices_offset.0.to_le_bytes())?;
        write_all(writer, &self.field_indices_count.to_le_bytes())?;

        write_all(writer, &self.list_indices_offset.0.to_le_bytes())?;
        write_all(writer, &self.list_indices_count.to_le_bytes())?;

        Ok(())
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

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct FileBinaryData {
    structs: Vec<u8>,
    fields: Vec<u8>,
    labels: Vec<u8>,
    field_data: Vec<u8>,
    field_indices: Vec<u8>,
    list_indices: Vec<u8>,
}
impl FileBinaryData {
    fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        write_all(writer, &self.structs)?;
        write_all(writer, &self.fields)?;
        write_all(writer, &self.labels)?;
        write_all(writer, &self.field_data)?;
        write_all(writer, &self.field_indices)?;
        write_all(writer, &self.list_indices)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Gff {
    file_type: String,
    file_version: String,
    structs: Vec<Struct>,
}
impl Gff {
    pub fn read(gff_data: impl Read + Seek, tlk_data: impl Read + Seek) -> Result<Self, Error> {
        let gff = GffData::read(gff_data)?;
        let tlk = Tlk::read(tlk_data)?;

        let structs = gff
            .structs
            .iter()
            .map(|s| s.resolve(&gff, &tlk))
            .collect_vec_result()?;

        Ok(Self {
            file_type: gff.header.file_type,
            file_version: gff.header.file_version,
            structs,
        })
    }

    pub fn write<W>(&self, file_type: &str, file_version: &str, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        // Store all the data in `vec`s then work out the offsets
        // when writing

        let mut binary_data = FileBinaryData::default();

        for s in self.structs.iter() {
            s.write(&mut binary_data)?;
        }

        fn increment_offset(offset: &mut i32, val: i32) -> Offset {
            (*offset) += val;
            Offset(*offset as u32)
        }

        let mut offset = 8;
        let struct_offset = increment_offset(&mut offset, 0);
        let struct_count = (binary_data.structs.len() as u32 / STRUCT_DATA_SIZE) as i32;

        let field_offset = increment_offset(&mut offset, binary_data.structs.len() as i32);
        let field_count = (binary_data.fields.len() as u32 / FIELD_DATA_SIZE) as i32;

        let label_offset = increment_offset(&mut offset, binary_data.fields.len() as i32);
        let label_count = (binary_data.labels.len() / LABEL_SIZE) as i32;

        let field_data_offset = increment_offset(&mut offset, binary_data.labels.len() as i32);
        let field_data_count = binary_data.field_data.len() as i32;

        let field_indices_offset =
            increment_offset(&mut offset, binary_data.field_data.len() as i32);
        let field_indices_count = (binary_data.field_indices.len() / size_of::<i32>()) as i32;

        let list_indices_offset =
            increment_offset(&mut offset, binary_data.field_indices.len() as i32);
        let list_indices_count = (binary_data.list_indices.len() / size_of::<i32>()) as i32;

        let header = Header {
            file_type: file_type.to_string(),
            file_version: file_version.to_string(),
            struct_offset,
            struct_count,
            field_offset,
            field_count,
            label_offset,
            label_count,
            field_data_offset,
            field_data_count,
            field_indices_offset,
            field_indices_count,
            list_indices_offset,
            list_indices_count,
        };

        header.write(writer)?;
        binary_data.write(writer)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn header_write_test() {
        let header = Header {
            file_type: "IFO ".to_string(),
            file_version: "V3.2".to_string(),
            struct_offset: Offset(1),
            struct_count: 2,
            field_offset: Offset(3),
            field_count: 4,
            label_offset: Offset(5),
            label_count: 6,
            field_data_offset: Offset(7),
            field_data_count: 8,
            field_indices_offset: Offset(9),
            field_indices_count: 10,
            list_indices_offset: Offset(11),
            list_indices_count: 12,
        };

        let mut buf = Cursor::new(vec![]);
        header.write(&mut buf).unwrap();
        buf.rewind().unwrap();

        let header_2 = Header::read(buf).unwrap();

        assert_eq!(header, header_2);
    }

    #[test]
    fn read_test() {
        let mut gff = Cursor::new(include_bytes!("../../tests/files/playerlist.ifo"));
        let mut tlk = Cursor::new(include_bytes!("../../tests/files/dialog.TLK"));

        let gff = Gff::read(&mut gff, &mut tlk).unwrap();

        for (i, s) in gff.structs.into_iter().enumerate() {
            println!("{i}: {s:#?}");
        }
    }

    #[test]
    fn write_test() {
        let mut gff = Cursor::new(include_bytes!("../../tests/files/playerlist.ifo"));
        let mut tlk = Cursor::new(include_bytes!("../../tests/files/dialog.TLK"));

        let gff = Gff::read(&mut gff, &mut tlk).unwrap();
        tlk.rewind().unwrap();

        let mut buf = Cursor::new(vec![]);
        gff.write(&gff.file_type, &gff.file_version, &mut buf)
            .unwrap();
        buf.rewind().unwrap();

        let gff_2 = Gff::read(&mut buf, &mut tlk).unwrap();

        assert_eq!(gff, gff_2)
    }
}
