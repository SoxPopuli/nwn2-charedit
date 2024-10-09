use super::{
    field::Field,
    label::{Label, LABEL_SIZE},
    FileBinaryData, GffData,
};
use crate::{
    error::{Error, IntoError},
    files::{gff::FieldData, tlk::Tlk, write_all},
};
use rust_utils::byte_readers::from_bytes_le;
use std::{
    collections::HashMap,
    io::{Read, Seek},
};

pub(crate) const STRUCT_DATA_SIZE: u32 = size_of::<StructData>() as u32;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructData {
    pub struct_type: i32,
    pub data_or_data_offset: i32,
    pub field_count: i32,
}

impl StructData {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let struct_type = from_bytes_le(&mut data).into_parse_error()?;
        let data_or_data_offset = from_bytes_le(&mut data).into_parse_error()?;
        let field_count = from_bytes_le(&mut data).into_parse_error()?;

        Ok(Self {
            struct_type,
            data_or_data_offset,
            field_count,
        })
    }

    pub fn get_field<'a>(&self, file: &'a GffData, index: i32) -> Option<&'a FieldData> {
        if index < 0 || index >= self.field_count {
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

    pub fn resolve<R>(&self, gff: &GffData, tlk: &Tlk<R>) -> Result<Struct, Error>
    where
        R: Read + Seek,
    {
        Struct::new(self, gff, tlk)
    }
}

/// *Warning*: duplicate labels possible?
#[derive(Debug, PartialEq)]
pub struct Struct {
    pub id: i32,
    pub fields: Vec<(Label, Field)>,
}
impl Struct {
    pub fn new<R>(s: &StructData, gff: &GffData, tlk: &Tlk<R>) -> Result<Self, Error>
    where
        R: Read + Seek,
    {
        let fields = (0..s.field_count)
            .map(|i| {
                let field = s
                    .get_field(gff, i)
                    .ok_or_else(|| Error::ParseError(format!("Field index {i} not found")))?;
                let label = field.get_label(&gff.labels);
                let field_data = field.get_data(gff, tlk)?;

                Ok::<_, Error>((label.clone(), field_data))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            id: s.struct_type,
            fields,
        })
    }

    /// Returns index into struct array
    pub(crate) fn write(&self, binary_data: &mut FileBinaryData) -> Result<u32, Error> {
        fn write_label(label: &Label, binary_data: &mut FileBinaryData) -> Result<i32, Error> {
            let offset = binary_data.labels.len();
            label.write(&mut binary_data.labels)?;
            Ok((offset / LABEL_SIZE) as i32)
        }

        let data_or_data_offset: i32 = if self.fields.len() == 1 {
            let offset = binary_data.fields.len();

            for (label, field) in self.fields.iter() {
                let label_index = write_label(label, binary_data)?;
                field.write_data(label_index, binary_data)?;
            }

            offset as i32
        } else {
            let base_offset = binary_data.field_indices.len();

            assert!(
                base_offset % 4 == 0,
                "Data index {} not aligned on u32 boundary",
                base_offset
            );

            for (label, field) in self.fields.iter() {
                let label_index = write_label(label, binary_data)?;
                let field_index = field.write_data(label_index, binary_data)?;

                binary_data
                    .field_indices
                    .extend_from_slice(&field_index.to_le_bytes());
            }

            base_offset as i32
        };

        let struct_data = StructData {
            struct_type: self.id,
            data_or_data_offset,
            field_count: self.fields.len() as i32,
        };

        let offset = binary_data.structs.len();
        write_all(
            &mut binary_data.structs,
            &struct_data.struct_type.to_le_bytes(),
        )?;
        write_all(
            &mut binary_data.structs,
            &struct_data.data_or_data_offset.to_le_bytes(),
        )?;
        write_all(
            &mut binary_data.structs,
            &struct_data.field_count.to_le_bytes(),
        )?;

        Ok(offset as u32 / STRUCT_DATA_SIZE)
    }
}
