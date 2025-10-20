use crate::{
    Tlk,
    error::Error,
    tlk_string_ref::TlkStringRef,
    two_d_array,
    ui::settings::{IconName, IconPath},
};
use iced::widget::image::Handle;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feat {
    pub label: String,
    pub name: TlkStringRef,
    pub desc: Option<TlkStringRef>,
    pub icon: Option<Handle>,
}

pub type FeatId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatRecord {
    pub feats: HashMap<FeatId, Feat>,
}
impl FeatRecord {
    pub fn new(
        tlk: &Tlk,
        reader: &mut two_d_array::FileReader2DA,
        icon_paths: &HashMap<IconName, IconPath>,
    ) -> Result<Self, Error> {
        let file_name = "feat.2da";

        let table = reader.read(file_name)?;

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

            let name = tlk
                .get_from_str_ref(name_ref)
                .expect("Couldn't find feat name in tlk file")?
                .to_string();

            let desc = desc_ref.and_then(|desc_ref| {
                let desc = tlk
                    .get_from_str_ref(desc_ref)
                    .expect("Couldn't find feat description in tlk file")?
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

            Some(Feat {
                label,
                name: TlkStringRef {
                    id: name_ref,
                    data: name,
                },
                desc: desc.map(|(desc_ref, desc)| TlkStringRef {
                    id: desc_ref,
                    data: desc,
                }),
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
