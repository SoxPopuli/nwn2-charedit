use std::collections::HashMap;

use crate::{
    feat::{Feat, FeatRecord},
    ids::class::Class,
    spell::{Spell, SpellRecord},
    ui::{HoverableEvent, HoverableState, hoverable},
};
use iced::{
    Length,
    widget::{
        Column, Image, button, column, container, horizontal_rule, horizontal_space, image::Handle,
        row, scrollable, text, text_input,
    },
};
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    HoverableEvent(HoverableEvent),
    TextChanged(String),
    Close,
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
    mode: SearchMode,
    search_text: String,
    hoverable_state: HoverableState,
}
impl State {
    pub fn is_active(&self) -> bool {
        match self.mode {
            SearchMode::Add | SearchMode::Swap(_) => true,
            SearchMode::None => false,
        }
    }

    pub fn open(&mut self, mode: SearchMode) {
        self.mode = mode;
    }

    fn close(&mut self) {
        self.mode = SearchMode::None;
        self.search_text.clear();
        self.hoverable_state.reset();
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::TextChanged(new_text) => {
                self.search_text = new_text;
                self.hoverable_state.reset();
            }
            Message::HoverableEvent(e) => e.update(&mut self.hoverable_state),
            Message::Close => self.close(),
        }
    }

    pub fn view<'a>(&self, kind: SearchKind<'a>) -> Element<'a> {
        let search_bar = text_input("Search...", &self.search_text).on_input(Message::TextChanged);

        let body: Element<'a> = match kind {
            SearchKind::Feats(record) => {
                let feats = record.feats.values();
                let search = self.search_text.to_ascii_lowercase();

                let elements = feats
                    .filter(|feat| feat.name.data.to_ascii_lowercase().contains(&search))
                    .enumerate()
                    .map(|(index, feat)| view_feat(feat, index, self.hoverable_state))
                    .intersperse_with(|| horizontal_rule(2).into());

                Column::from_iter(elements).width(Length::Fill).into()
            }
            SearchKind::Spells {
                spell_record,
                class,
                level,
            } => {
                let spells = spell_record.get_spells_per_class_level(class);
                let spells = spells.get(level as usize).and_then(|x| x.as_ref());

                let elements = match spells {
                    Some(spells) => spells
                        .iter()
                        .enumerate()
                        .map(|(index, spell)| view_spell(spell, index, self.hoverable_state))
                        .intersperse_with(|| horizontal_rule(2).into())
                        .collect(),

                    None => vec![],
                };

                let elements = iced::widget::Column::from_vec(elements);

                elements.into()
            }
        };

        let body = scrollable(body).height(Length::Fill);

        let footer = row![
            horizontal_space().width(Length::Fill),
            button("Close").on_press(Message::Close),
            button("Select")
        ]
        .height(Length::Fixed(32.0))
        .spacing(16);

        crate::ui::bordered_padded(
            column![search_bar, body, container(footer).padding(16)].spacing(8.0),
        )
        .into()
    }
}

fn view_feat(feat: &Feat, index: usize, hoverable_state: HoverableState) -> Element<'static> {
    let icon: Element<'_> = match &feat.icon {
        Some(icon) => Image::new(icon).into(),
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

    hoverable(item, index, hoverable_state, Message::HoverableEvent)
        .width(Length::Fill)
        .into()
}

fn view_spell(spell: &Spell, index: usize, hoverable_state: HoverableState) -> Element<'static> {
    let icon: Element<'_> = match &spell.icon {
        Some(handle) => Image::<Handle>::new(handle).width(40).height(40).into(),
        None => horizontal_space().width(40).into(),
    };

    let name = spell.name.data.clone();

    let desc = match spell.desc.as_ref() {
        Some(desc) => desc.data.as_str(),
        None => "",
    }
    .to_string();

    let item = row![icon, text(name).width(120), text(desc)]
        .width(Length::Fill)
        .spacing(16)
        .padding(16);

    hoverable(item, index, hoverable_state, Message::HoverableEvent)
        .width(Length::Fill)
        .into()
}
