use crate::error::{Error, IntoError};
use crate::files::gff::Field;
use rust_utils::byte_readers::from_bytes_le;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct Struct {
    pub struct_type: i32,
    pub data_or_data_offset: i32,
    pub field_count: i32,
}

impl Struct {
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

    pub fn get_field<'a>(&self, fields: &'a [Field], index: i32) -> Option<&'a Field> {
        if index < 0 || index >= self.field_count {
            return None;
        }

        if self.field_count == 1 {
            // Index into field array
            let field = &fields[self.data_or_data_offset as usize];
            Some(field)
        } else {
            // Byte offset into field indices
            assert!(self.data_or_data_offset % 4 == 0);

            let index = self.data_or_data_offset / 4;
            let field = &fields[index as usize];

            Some(field)
        }
    }
}
