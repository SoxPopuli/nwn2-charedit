use std::{
    collections::HashMap,
    path::Path,
};

use iced::widget::image::Handle;

use crate::{
    Tlk,
    error::Error,
    tlk_string_ref::TlkStringRef,
    ui::settings::{IconName, IconPath},
};

type SpellLevel = Option<std::num::NonZeroU8>;

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

pub type SpellId = usize;

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

            let name = tlk.get_from_str_ref(name_ref).ok().flatten()?.to_string();

            let desc = desc_ref.and_then(|desc_ref| {
                let desc = tlk
                    .get_from_str_ref(desc_ref)
                    .expect("Couldn't find spell description in tlk file")?
                    .to_string();
                Some((desc_ref, desc))
            });

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
                name: TlkStringRef {
                    id: name_ref,
                    data: name,
                },
                desc: desc.map(|(desc_ref, desc)| TlkStringRef {
                    id: desc_ref,
                    data: desc,
                }),
                label,
                icon,
                spell_levels,
            })
        };

        let spells = table
            .data
            .row_iter()
            .enumerate()
            .filter_map(|(i, r)| from_row(r).map(|x| (i, x)))
            .collect();

        Ok(Self { spells })
    }
}
