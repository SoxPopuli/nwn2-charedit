use std::{collections::HashMap, path::Path};

use iced::widget::image::Handle;

use crate::{
    Tlk,
    error::Error,
    ids::class::Class,
    tlk_string_ref::TlkStringRef,
    ui::settings::{IconName, IconPath},
};

type SpellLevel = Option<u8>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SpellLevels {
    pub bard: SpellLevel,
    pub cleric: SpellLevel,
    pub druid: SpellLevel,
    pub paladin: SpellLevel,
    pub ranger: SpellLevel,
    pub wiz_sorc: SpellLevel,
    pub warlock: SpellLevel,
    pub innate: SpellLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spell {
    pub label: String,
    pub name: TlkStringRef,
    pub desc: Option<TlkStringRef>,
    pub icon: Option<Handle>,
    pub spell_levels: SpellLevels,
}

pub type SpellId = u16;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SpellRecord {
    pub spells: HashMap<SpellId, Spell>,
}
impl SpellRecord {
    pub fn new(
        tlk: &Tlk,
        game_dir: &Path,
        icon_paths: &HashMap<IconName, IconPath>,
    ) -> Result<Self, Error> {
        let file_name = "spells.2da";

        let file_path = super::join_path(
            game_dir,
            &["campaigns", "westgate_campaign", "2da", file_name],
        );

        let table = {
            let f = std::fs::File::open(file_path)?;
            let reader = std::io::BufReader::new(f);
            nwn_lib::files::two_da::parse(reader)?
        };

        let [
            label_idx,
            name_idx,
            desc_idx,
            icon_idx,
            bard_idx,
            cleric_idx,
            druid_idx,
            paladin_idx,
            ranger_idx,
            wiz_sorc_idx,
            warlock_idx,
            innate_idx,
        ] = table
            .find_column_indices([
                "Label",
                "Name",
                "SpellDesc",
                "IconResRef",
                "Bard",
                "Cleric",
                "Druid",
                "Paladin",
                "Ranger",
                "Wiz_Sorc",
                "Warlock",
                "Innate",
            ])
            .map_err(|e| Error::MissingTableColumn {
                file: file_name,
                column: e,
            })?;

        let from_row = |row: &[Option<String>]| -> Option<Spell> {
            let label = row.get(label_idx)?.clone()?;

            let name_ref = row.get(name_idx)?.as_deref()?;
            let name_ref = name_ref.parse().ok()?;

            let desc_ref = row.get(desc_idx)?.as_deref()?;
            let desc_ref = desc_ref.parse().ok();

            let icon = row
                .get(icon_idx)?
                .as_deref()
                .and_then(|name| icon_paths.get(name))
                .and_then(|path| {
                    let f = std::fs::File::open(path).ok()?;
                    let reader = std::io::BufReader::new(f);
                    dds::Dds::read(reader).ok()
                })
                .map(|dds| {
                    let pixels = Vec::from_iter(
                        dds.pixels
                            .into_iter()
                            .flat_map(|dds::Rgba { r, g, b, a }| [r, g, b, a]),
                    );
                    Handle::from_rgba(dds.header.width, dds.header.height, pixels)
                });

            let get_spell_level = |idx: usize| {
                row.get(idx)
                    .and_then(|x| x.as_deref())
                    .and_then(|x| x.parse().ok())
            };

            let spell_levels = SpellLevels {
                bard: get_spell_level(bard_idx),
                cleric: get_spell_level(cleric_idx),
                druid: get_spell_level(druid_idx),
                paladin: get_spell_level(paladin_idx),
                ranger: get_spell_level(ranger_idx),
                wiz_sorc: get_spell_level(wiz_sorc_idx),
                warlock: get_spell_level(warlock_idx),
                innate: get_spell_level(innate_idx),
            };

            Some(Spell {
                name: TlkStringRef::from_id(tlk, name_ref).ok()?,
                desc: desc_ref.and_then(|r| TlkStringRef::from_id(tlk, r).ok()),
                label,
                icon,
                spell_levels,
            })
        };

        let spells = table
            .data
            .row_iter()
            .enumerate()
            .filter_map(|(i, r)| from_row(r).map(|x| (i as u16, x)))
            .collect();

        Ok(Self { spells })
    }

    pub fn get_spells_for_class(&self, class: Class) -> Option<Vec<&Spell>> {
        macro_rules! for_class {
            ($class:ident) => {
                Some(
                    self.spells
                        .values()
                        .filter(|x| x.spell_levels.$class.is_some())
                        .collect(),
                )
            };
        }

        match class {
            Class::Bard => for_class!(bard),
            Class::Cleric => for_class!(cleric),
            Class::Druid => for_class!(druid),
            Class::Paladin => for_class!(paladin),
            Class::Ranger => for_class!(ranger),
            Class::Wizard | Class::Sorcerer => for_class!(wiz_sorc),
            Class::Warlock => for_class!(warlock),
            _ => None,
        }
    }

    fn for_spells_for_class(&self, class: Class, f: impl FnMut(&Spell)) {
        macro_rules! for_class {
            ($class:ident) => {
                self.spells
                    .values()
                    .filter(|x| x.spell_levels.$class.is_some())
                    .for_each(f)
            };
        }

        match class {
            Class::Bard => for_class!(bard),
            Class::Cleric => for_class!(cleric),
            Class::Druid => for_class!(druid),
            Class::Paladin => for_class!(paladin),
            Class::Ranger => for_class!(ranger),
            Class::Wizard | Class::Sorcerer => for_class!(wiz_sorc),
            Class::Warlock => for_class!(warlock),
            _ => (),
        }
    }

    pub fn get_spells_per_class_level<'a>(&'a self, class: Class) -> [Option<Vec<&'a Spell>>; 10] {
        let mut spells_per_level: [Option<Vec<&'a Spell>>; 10] = [const { None }; 10];

        let mut set_spell = |spell, level| match &mut spells_per_level[level as usize] {
            Some(spells) => {
                spells.push(spell);
            }
            s @ None => {
                *s = Some(vec![spell]);
            }
        };

        for s in self.spells.values() {
            macro_rules! set_spells_for_class {
                ($class:ident) => {
                    if let Some(lvl) = s.spell_levels.$class {
                        set_spell(s, lvl)
                    }
                };
            }

            match class {
                Class::Bard => set_spells_for_class!(bard),
                Class::Cleric => set_spells_for_class!(cleric),
                Class::Druid => set_spells_for_class!(druid),
                Class::Paladin => set_spells_for_class!(paladin),
                Class::Ranger => set_spells_for_class!(ranger),
                Class::Wizard | Class::Sorcerer => set_spells_for_class!(wiz_sorc),
                Class::Warlock => set_spells_for_class!(warlock),
                _ => {}
            }
        }

        spells_per_level
    }
}
