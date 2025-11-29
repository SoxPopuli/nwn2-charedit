use crate::{
    error::Error,
    field_ref::FieldRef,
    ids::{class::Class, spell::Spell},
};
use nwn_lib::files::gff::{
    field::{Field, LabeledField},
    label::Label,
    r#struct::{Struct, StructField},
};
use std::fmt::Display;

fn opt_field<T>(
    x: Option<T>,
    field_name: impl Display,
    class_name: impl Display,
) -> Result<T, Error> {
    x.ok_or_else(|| Error::MissingField(format!("{} in {}", field_name, class_name)))
}

#[derive(Debug, Clone)]
pub struct SpellKnownList {
    pub list_ref: StructField,
    pub spells: Vec<Spell>,
}
impl SpellKnownList {
    pub fn new(list_field: StructField) -> Result<Self, Error> {
        let lock = list_field.read()?;
        let list = lock.field.expect_list()?;

        let spells = list
            .iter()
            .map(|x| {
                let field = &x.fields[0];
                field.read_field(Field::expect_word).map(Spell)
            })
            .collect::<Result<Vec<_>, _>>()?;

        drop(lock);

        Ok(Self {
            list_ref: list_field,
            spells,
        })
    }

    fn create_spell_struct(Spell(spell): Spell) -> Struct {
        let label = Label::from_string("Spell");
        let field = StructField::new(LabeledField {
            label,
            field: Field::Word(spell),
        });

        Struct {
            id: 3,
            original_data_or_data_offset: u32::MAX,
            fields: vec![field],
        }
    }

    pub fn add_spell(&mut self, spell: Spell) {
        let mut field_lock = self.list_ref.write().unwrap();

        match &mut field_lock.field {
            Field::List(lst) => {
                let s = Self::create_spell_struct(spell);

                lst.push(s);
            }
            x => panic!("Unexpected field: {x:?}"),
        }

        self.spells.push(spell);
    }

    pub fn swap_spell(&mut self, index: usize, new_id: Spell) {
        let mut field_lock = self.list_ref.write().unwrap();

        let new_spell = Self::create_spell_struct(new_id);

        match &mut field_lock.field {
            Field::List(lst) => {
                if let Some(old_spell) = lst.get_mut(index) {
                    *old_spell = new_spell.clone();
                }
            }
            x => panic!("Unexpected field: {x:?}"),
        }

        if let Some(old_ref) = self.spells.get_mut(index) {
            *old_ref = new_id;
        }
    }

    pub fn remove_spell(&mut self, index: usize) {
        let mut field_lock = self.list_ref.write().unwrap();

        match &mut field_lock.field {
            Field::List(lst) => {
                lst.remove(index);
            }
            x => panic!("Unexpected field: {x:?}"),
        }

        self.spells.remove(index);
    }
}

#[derive(Debug, Clone)]
pub struct PlayerClass {
    pub class: FieldRef<Class>,
    pub level: FieldRef<i16>,

    pub is_caster: bool,
    pub spell_known_list: [Option<SpellKnownList>; 10],
}
impl PlayerClass {
    pub fn new(s: &Struct) -> Result<Self, Error> {
        let mut class = None;
        let mut level = None;
        let mut is_caster = false;

        let mut known_list = [const { None }; 10];

        for f in &s.fields {
            let field_lock = f.read()?;
            match field_lock.label.as_str() {
                "Class" => {
                    let field_ref = FieldRef::new(f.clone(), |f| f.expect_int().map(Class))?;
                    class = Some(field_ref);
                }

                "ClassLevel" => {
                    level = Some(FieldRef::new(f.clone(), Field::expect_short)?);
                }

                label @ ("KnownList0" | "KnownList1" | "KnownList2" | "KnownList3"
                | "KnownList4" | "KnownList5" | "KnownList6" | "KnownList7"
                | "KnownList8" | "KnownList9") => {
                    is_caster = true;

                    let spell_level: usize = label[9..]
                        .parse()
                        .map_err(|e: std::num::ParseIntError| Error::ParseError(e.to_string()))?;
                    let spell_structs = field_lock.field.expect_list()?;

                    let spells = spell_structs
                        .iter()
                        .map(|x| {
                            let spell = &x.fields[0];
                            spell.read_field(Field::expect_word).map(Spell)
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    let known = SpellKnownList {
                        list_ref: f.clone(),
                        spells,
                    };
                    known_list[spell_level] = Some(known);
                }

                _ => {}
            }
        }

        macro_rules! opt {
            ($x:expr, $field_name:expr) => {
                opt_field($x, $field_name, "PlayerClass")
            };
        }

        Ok(Self {
            class: opt!(class, "Class")?,
            level: opt!(level, "ClassLevel")?,
            is_caster,
            spell_known_list: known_list,
        })
    }
}
