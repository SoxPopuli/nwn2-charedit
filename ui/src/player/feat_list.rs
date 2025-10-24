use nwn_lib::files::gff::{field::Field, r#struct::StructField};

use crate::{error::Error, field_ref::FieldRef};

type FeatId = u16;

#[derive(Debug, Clone)]
pub struct FeatList {
    pub list_ref: FieldRef<Vec<FieldRef<FeatId>>>,
}
impl FeatList {
    pub fn from_field(list: StructField) -> Result<Self, Error> {
        FieldRef::new(list, |f| {
            f.expect_list().map_err(Error::LibError).and_then(|lst| {
                lst.iter()
                    .filter_map(|s| s.fields.first())
                    .map(|field| FieldRef::new(field.clone(), Field::expect_word))
                    .collect::<Result<Vec<_>, _>>()
            })
        })
        .map(|x| Self { list_ref: x })
    }
}
