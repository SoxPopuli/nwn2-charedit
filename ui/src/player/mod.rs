pub mod feat_list;
pub mod player_class;

use crate::{Tlk, error::Error, field_ref::FieldRef, player::feat_list::FeatList, two_d_array};
use nwn_lib::files::gff::{field::Field, r#struct::Struct};
pub use player_class::PlayerClass;

macro_rules! make_builder {
    (struct $name: ident { $($field: ident : $t: ty),+ $(,)* }) => {
        #[derive(Debug, Default)]
        pub struct $name {
            $(pub $field : Option<$t>),+
        }
        impl $name {
            $(
                pub fn $field (&mut self, x: $t) {
                    self.$field = Some(x);
                }
            )+
        }
    };
}

common::open_enum! {
    pub enum Gender: u8 {
        Male = 0,
        Female = 1,
    }
}

#[derive(Debug, Clone)]
pub struct Attributes {
    pub str: FieldRef<u8>,
    pub dex: FieldRef<u8>,
    pub con: FieldRef<u8>,
    pub int: FieldRef<u8>,
    pub wis: FieldRef<u8>,
    pub cha: FieldRef<u8>,
}

#[derive(Debug, Clone)]
pub struct Race {
    pub race: String,
    pub subrace: Option<String>,
}
impl std::fmt::Display for Race {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.subrace.as_deref() {
            Some(subrace) => f.write_str(subrace),
            None => f.write_str(&self.race),
        }
    }
}

fn get_race_name_from_id(
    tlk: &Tlk,
    reader: &mut two_d_array::FileReader2DA,
    field: &Field,
) -> Result<String, Error> {
    let file_name = "racialtypes.2da";
    let table = reader.read(file_name)?;
    let race_id = field.expect_byte()?;
    let name_idx = table
        .find_column_index("Name")
        .ok_or(Error::MissingField(format!(
            "Missing 'Name' field in {file_name}"
        )))?;

    let s_ref = table.data[(name_idx, race_id as usize)]
        .clone()
        .ok_or(Error::MissingField(format!(
            "Missing race name in {file_name} for {race_id}"
        )))?;

    let x = tlk
        .get_from_str_ref(
            s_ref
                .parse()
                .map_err(|e: std::num::ParseIntError| Error::MissingField(e.to_string()))?,
        )
        .map_err(Error::LibError)
        .and_then(|x| x.ok_or(Error::MissingField("Missing race name str_ref".into())))?;

    Ok(x.to_string())
}

fn get_subrace_name_from_id(
    tlk: &Tlk,
    reader: &mut two_d_array::FileReader2DA,
    field: &Field,
) -> Result<String, Error> {
    let file_name = "racialsubtypes.2da";
    let table = reader.read(file_name)?;
    let subrace_id = field.expect_byte()?;

    let name_idx = table
        .find_column_index("Name")
        .ok_or(Error::MissingField(format!(
            "Missing 'Name' field in {file_name}"
        )))?;

    let s_ref = table.data[(name_idx, subrace_id as usize)]
        .clone()
        .ok_or(Error::MissingField(format!(
            "Missing race name in {file_name} for {subrace_id}"
        )))?;

    let x = tlk
        .get_from_str_ref(
            s_ref
                .parse()
                .map_err(|e: std::num::ParseIntError| Error::MissingField(e.to_string()))?,
        )
        .map_err(Error::LibError)
        .and_then(|x| x.ok_or(Error::MissingField("Missing race name str_ref".into())))?;

    Ok(x.to_string())
}

#[derive(Debug, Clone)]
pub struct Alignment {
    pub good_evil: FieldRef<u8>,
    pub lawful_chaotic: FieldRef<u8>,
}
impl std::fmt::Display for Alignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let good_evil = self.good_evil.get();
        let lawful_chaotic = self.lawful_chaotic.get();

        let good_evil = match good_evil {
            70..=100 => Some("Good"),
            31..=69 => Some("Neutral"),
            0..=30 => Some("Evil"),
            _ => None,
        };

        let lawful_chaotic = match lawful_chaotic {
            70..=100 => Some("Lawful"),
            31..=69 => Some("Neutral"),
            0..=30 => Some("Chaotic"),
            _ => None,
        };

        match (good_evil, lawful_chaotic) {
            (Some(ge), Some(lc)) => write!(f, "{lc} {ge}"),
            (Some(ge), None) => write!(f, "{ge}"),
            (None, Some(lc)) => write!(f, "{lc}"),
            (None, None) => write!(f, "Unknown"),
        }
    }
}

make_builder! {
    struct PlayerBuilder {
        first_name: FieldRef<String>,
        last_name: FieldRef<String>,
        gender: FieldRef<Gender>,
        race: FieldRef<String>,
        subrace: FieldRef<String>,
        classes: Vec<player_class::PlayerClass>,
        str: FieldRef<u8>,
        dex: FieldRef<u8>,
        con: FieldRef<u8>,
        int: FieldRef<u8>,
        wis: FieldRef<u8>,
        cha: FieldRef<u8>,
        good_evil: FieldRef<u8>,
        lawful_chaotic: FieldRef<u8>,
        feats: FeatList,
    }
}

impl PlayerBuilder {
    fn build(self) -> Result<Player, Error> {
        macro_rules! unwrap_field {
            ($field: ident) => {
                self.$field
                    .ok_or($crate::error::Error::MissingField(format!(
                        "Missing field {} in player builder",
                        stringify!($field)
                    )))?
            };
        }

        Ok(Player {
            first_name: unwrap_field!(first_name),
            last_name: unwrap_field!(last_name),
            race: Race {
                race: unwrap_field!(race).value,
                subrace: self.subrace.map(|x| x.value),
            },
            classes: unwrap_field!(classes),
            gender: unwrap_field!(gender).value,
            attributes: Attributes {
                str: unwrap_field!(str),
                dex: unwrap_field!(dex),
                con: unwrap_field!(con),
                int: unwrap_field!(int),
                wis: unwrap_field!(wis),
                cha: unwrap_field!(cha),
            },
            alignment: Alignment {
                good_evil: unwrap_field!(good_evil),
                lawful_chaotic: unwrap_field!(lawful_chaotic),
            },
            feats: unwrap_field!(feats),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub first_name: FieldRef<String>,
    pub last_name: FieldRef<String>,
    pub race: Race,
    pub gender: Gender,
    pub classes: Vec<PlayerClass>,
    pub attributes: Attributes,
    pub alignment: Alignment,
    pub feats: FeatList,
}

impl Player {
    pub fn new(
        tlk: &Tlk,
        data_reader: &mut two_d_array::FileReader2DA,
        player_struct: &Struct,
    ) -> Result<Self, Error> {
        let read_name = |field: &Field| -> Result<String, Error> {
            let s = field.expect_exolocstring()?;
            Ok(s.substrings
                .iter()
                .map(|sub| &sub.data)
                .fold(String::new(), |acc, x| acc + x))
        };

        let mut player_builder = PlayerBuilder::default();

        for field in &player_struct.fields {
            let lock = field.read()?;
            let label = &lock.label;

            macro_rules! read_field {
                ($builder_fn:ident, $expect_fn:expr) => {{ player_builder.$builder_fn(FieldRef::new(field.clone(), $expect_fn)?) }};
            }

            match label.as_str() {
                "FirstName" => read_field!(first_name, read_name),
                "LastName" => read_field!(last_name, read_name),
                "Race" => read_field!(race, |f| get_race_name_from_id(tlk, data_reader, f)),
                "Gender" => read_field!(gender, |f| { Field::expect_byte(f).map(Gender) }),
                "Subrace" => {
                    read_field!(subrace, |f| get_subrace_name_from_id(tlk, data_reader, f))
                }
                "Str" => read_field!(str, Field::expect_byte),
                "Dex" => read_field!(dex, Field::expect_byte),
                "Con" => read_field!(con, Field::expect_byte),
                "Int" => read_field!(int, Field::expect_byte),
                "Wis" => read_field!(wis, Field::expect_byte),
                "Cha" => read_field!(cha, Field::expect_byte),
                "GoodEvil" => read_field!(good_evil, Field::expect_byte),
                "LawfulChaotic" => read_field!(lawful_chaotic, Field::expect_byte),
                "LvlStatList" => {
                    // let lock = field.read().unwrap();
                    // let s = lock.field.expect_list().unwrap();
                }
                "ClassList" => {
                    let lock = field.read()?;
                    let list = lock.field.expect_list()?;

                    let classes = list
                        .iter()
                        .map(PlayerClass::new)
                        .collect::<Result<Vec<_>, _>>()?;

                    player_builder.classes(classes);
                }
                "FeatList" => {
                    let feats = FeatList::from_field(field.clone())?;
                    player_builder.feats(feats);
                }

                _ => {}
            }
        }

        player_builder.build()
    }
}
