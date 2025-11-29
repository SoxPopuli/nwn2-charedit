use crate::{
    feat::{Feat, FeatId, FeatRecord},
    ids::class::Class,
    spell::{Spell, SpellId, SpellRecord},
    ui::{HoverableEvent, HoverableState, hoverable},
};
use iced::{
    Length,
    widget::{
        Column, Image, button, column, container, horizontal_rule, horizontal_space, row,
        scrollable, text, text_input,
    },
};
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    HoverableEvent((usize, HoverableEvent)),
    TextChanged(String),
    Close,
    Confirm,
}

type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchKind<'a> {
    Feats(&'a FeatRecord),
    Spells {
        spell_record: &'a SpellRecord,
        class: Class,
        level: u8,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    #[default]
    None,
    Add,
    Swap(usize),
}

#[derive(Default)]
pub struct State {
    pub mode: SearchMode,
    pub search_text: String,
    pub hoverable_state: HoverableState,
    pub selected_id: Option<usize>,
}
impl State {
    pub fn is_active(&self) -> bool {
        matches!(self.mode, SearchMode::Add | SearchMode::Swap(_))
    }

    pub fn open(&mut self, mode: SearchMode) {
        self.mode = mode;
    }

    fn close(&mut self) {
        self.mode = SearchMode::None;
        self.search_text.clear();
        self.hoverable_state.reset();
        self.selected_id = None;
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::TextChanged(new_text) => {
                self.search_text = new_text;
                self.hoverable_state.reset();
            }
            Message::HoverableEvent((id, e)) => {
                if let HoverableEvent::EntrySelected(_) = e {
                    self.selected_id = Some(id);
                }

                e.update(&mut self.hoverable_state);
            }
            Message::Close => self.close(),
            Message::Confirm => self.close(),
        }
    }

    fn view_feats<'a>(
        &self,
        feats: impl Iterator<Item = (FeatId, &'a Feat)>,
    ) -> Column<'a, Message> {
        let elements = feats
            .enumerate()
            .map(|(index, (feat_id, feat))| view_feat(feat_id, feat, index, self.hoverable_state))
            .intersperse_with(|| horizontal_rule(2).into());

        Column::from_iter(elements).width(Length::Fill)
    }

    fn view_spells<'a>(
        &self,
        spells: impl Iterator<Item = (SpellId, &'a Spell)>,
    ) -> Column<'a, Message> {
        let elements = spells
            .into_iter()
            .enumerate()
            .map(|(index, (id, spell))| view_spell(id, spell, index, self.hoverable_state))
            .intersperse_with(|| horizontal_rule(2).into())
            .collect();

        let elements = iced::widget::Column::from_vec(elements);

        elements.width(Length::Fill)
    }

    pub fn view<'a>(&self, kind: SearchKind<'a>) -> Element<'a> {
        let search_bar = text_input("Search...", &self.search_text).on_input(Message::TextChanged);

        let body: Element<'a> = match kind {
            SearchKind::Feats(record) => {
                let feats = record.feats.iter().map(|(id, feat)| (*id, feat));

                if self.search_text.len() < 3 {
                    Column::new()
                } else {
                    let search = self.search_text.to_ascii_lowercase();
                    self.view_feats(feats.filter(|(_id, feat)| {
                        feat.name.data.to_ascii_lowercase().contains(&search)
                    }))
                }
                .into()
            }
            SearchKind::Spells {
                spell_record,
                class,
                level,
            } => {
                let spells = spell_record.get_spells_per_class_level(class);
                let spells = spells
                    .get(level as usize)
                    .and_then(|x| x.as_ref())
                    .into_iter()
                    .flatten()
                    .map(|(id, spell)| (*id, *spell));

                if self.search_text.is_empty() {
                    self.view_spells(spells).into()
                } else {
                    let search = self.search_text.to_ascii_lowercase();
                    let spells = spells.filter(|(_id, spell)| {
                        spell.name.data.to_ascii_lowercase().contains(&search)
                    });

                    self.view_spells(spells).into()
                }
            }
        };

        let body = scrollable(body).height(Length::Fill);

        let footer = row![
            horizontal_space().width(Length::Fill),
            button("Close").on_press(Message::Close),
            button("Select").on_press_maybe(self.selected_id.map(|_| Message::Confirm)),
        ]
        .height(Length::Fixed(32.0))
        .spacing(16);

        crate::ui::bordered_padded(
            column![search_bar, body, container(footer).padding(16)].spacing(8.0),
        )
        .into()
    }
}

fn view_feat(
    feat_id: FeatId,
    feat: &Feat,
    index: usize,
    hoverable_state: HoverableState,
) -> Element<'static> {
    let icon: Element<'_> = match &feat.icon {
        Some(icon) => Image::new(icon).width(40).height(40).into(),
        None => horizontal_space().width(40).into(),
    };

    let name = feat.name.data.clone();

    let desc = feat
        .desc
        .as_ref()
        .map(|x| x.data.as_str())
        .unwrap_or_default()
        .to_string();

    let item = row![icon, text(name).width(120), text(desc),]
        .width(Length::Fill)
        .padding(16)
        .spacing(16);

    hoverable(item, index, hoverable_state, |evt| {
        Message::HoverableEvent((feat_id, evt))
    })
    .width(Length::Fill)
    .into()
}

fn view_spell(
    spell_id: SpellId,
    spell: &Spell,
    index: usize,
    hoverable_state: HoverableState,
) -> Element<'static> {
    let icon: Element<'_> = match &spell.icon {
        Some(handle) => Image::new(handle).width(40).height(40).into(),
        None => horizontal_space().width(40).into(),
    };

    let name = spell.name.data.clone();

    let desc = match spell.desc.as_ref() {
        Some(desc) => desc.data.as_str(),
        None => "",
    }
    .to_string();

    let item = row![icon, text(name).width(120), text(desc)]
        // .width(Length::Fill)
        .spacing(16)
        .padding(16);

    hoverable(item, index, hoverable_state, |evt| {
        Message::HoverableEvent((spell_id, evt))
    })
    // .width(Length::Fill)
    .into()
}
