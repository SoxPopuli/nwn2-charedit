use super::{
    bin::{Gff as BinGff, Struct as BinStruct},
    field::LabeledField,
};
use crate::{
    error::Error,
    files::tlk::Tlk,
};
use std::io::{Read, Seek};

/// *Warning*: duplicate labels possible?
#[derive(Debug, PartialEq)]
pub struct Struct {
    pub id: u32,
    pub fields: Vec<LabeledField>,
}
impl Struct {
    pub fn new<R>(s: &BinStruct, gff: &BinGff, tlk: &Tlk<R>) -> Result<Self, Error>
    where
        R: Read + Seek,
    {
        let fields = (0..s.field_count)
            .map(|i| {
                let field = s
                    .get_field(gff, i)
                    .ok_or_else(|| Error::ParseError(format!("Field index {i} not found")))?;

                let label = gff.labels[field.label_index as usize].clone();
                let field_data = field.to_field(gff, tlk)?;

                let labeled_field = LabeledField {
                    label: label.clone(),
                    field: field_data,
                };

                Ok::<_, Error>(labeled_field)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { id: s.id, fields })
    }
}
