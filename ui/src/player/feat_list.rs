use nwn_lib::files::gff::{
    field::{Field, LabeledField},
    label::Label,
    r#struct::{Struct, StructField},
};

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

    fn create_feat_struct(feat: FeatId) -> Struct {
        let label = Label::from_string("Feat");
        let field = StructField::new(LabeledField {
            label,
            field: Field::Word(feat),
        });

        Struct {
            id: 0,
            original_data_or_data_offset: u32::MAX,
            fields: vec![field],
        }
    }

    pub fn add_feat(&mut self, feat: FeatId) {
        let mut field_lock = self.list_ref.field.write().unwrap();

        let s = Self::create_feat_struct(feat);

        match &mut field_lock.field {
            Field::List(lst) => {
                lst.push(Self::create_feat_struct(feat));
            }
            x => panic!("Unexpected field: {x:?}"),
        };

        let field_ref = FieldRef::new(s.fields[0].clone(), Field::expect_word).unwrap();
        self.list_ref.value.push(field_ref);
    }

    pub fn remove_feat(&mut self, index: usize) {
        let mut lock = self.list_ref.field.write().unwrap();

        match &mut lock.field {
            Field::List(lst) => {
                lst.remove(index);
            }
            x => panic!("Unexpected field: {x:?}"),
        };

        self.list_ref.value.remove(index);
    }
}
