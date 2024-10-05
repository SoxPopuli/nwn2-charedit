use crate::error::{Error, IntoError};
use crate::files::gff::FieldData;
use crate::files::tlk::Tlk;
use rust_utils::byte_readers::from_bytes_le;
use rust_utils::collect_vec::CollectVecResult;
use std::io::{Read, Seek};

use super::field::LabeledField;
use super::GffData;

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
                "Data index not aligned on u32 boundary :("
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

#[derive(Debug, PartialEq)]
pub struct Struct {
    pub fields: Vec<LabeledField>,
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
                let data = field.get_data(gff, tlk)?;

                Ok::<_, Error>(LabeledField {
                    field: data,
                    label: label.clone(),
                })
            })
            .collect_vec_result()?;

        Ok(Self { fields })
    }
}
