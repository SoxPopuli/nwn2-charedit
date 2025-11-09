use crate::{
    Tlk,
    error::Error,
    tlk_string_ref::TlkStringRef,
    ui::settings::{IconName, IconPath},
};
use iced::widget::image::Handle;
use nwn_lib::files::two_da;
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feat {
    pub label: String,
    pub name: TlkStringRef,
    pub desc: Option<TlkStringRef>,
    pub icon: Option<Handle>,
}

pub type FeatId = usize;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FeatRecord {
    pub feats: HashMap<FeatId, Feat>,
}
impl FeatRecord {
    pub fn new(
        tlk: &Tlk,
        game_dir: &Path,
        icon_paths: &HashMap<IconName, IconPath>,
    ) -> Result<Self, Error> {
        let file_name = "feat.2da";
        let file_path = super::join_path(
            game_dir,
            &["campaigns", "westgate_campaign", "2da", file_name],
        );

        let table = {
            let file = File::open(file_path)?;
            let reader = BufReader::new(file);
            two_da::parse(reader)?
        };

        let [label_idx, name_idx, desc_idx, icon_idx] = table
            .find_column_indices(["LABEL", "FEAT", "DESCRIPTION", "ICON"])
            .map_err(|e| Error::MissingTableColumn {
                file: file_name,
                column: e,
            })?;

        let from_row = |row: &[Option<String>]| -> Option<Feat> {
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

            Some(Feat {
                label,
                name: TlkStringRef::from_id(tlk, name_ref).ok()?,
                desc: desc_ref.and_then(|r| TlkStringRef::from_id(tlk, r).ok()),
                icon,
            })
        };

        let feats = table
            .data
            .row_iter()
            .enumerate()
            .filter_map(|(i, x)| from_row(x).map(|x| (i, x)))
            .collect();

        Ok(Self { feats })
    }
}
