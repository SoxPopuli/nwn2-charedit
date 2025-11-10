#![allow(unstable_name_collisions)]

use crate::{
    player::{Player, PlayerClass},
    spell::SpellRecord,
    ui::{HoverableEvent, HoverableState, hoverable},
};
use iced::{
    Length,
    widget::{
        Column, Image, column, container, horizontal_rule, image::Handle, row, scrollable, text,
        vertical_space,
    },
};
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    HoverableEvent(HoverableEvent),
}

pub type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Default, Clone)]
pub struct State {
    hoverable_state: HoverableState,
}
impl State {
    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::HoverableEvent(e) => e.update(&mut self.hoverable_state),
        }
    }
    fn view_class<'a>(
        &'a self,
        class: &'a PlayerClass,
        spell_record: &'a SpellRecord,
    ) -> Element<'a> {
        let class_name = class.class.get().to_string();
        let class_name = container(text(class_name).size(20)).padding(16);

        let spells = class
            .spell_known_list
            .iter()
            .flatten()
            .flat_map(|lst| lst.spells.iter())
            .filter_map(|spell| {
                let spell = spell_record.spells.get(&(spell.0 as usize))?;

                let icon: Element<'_> = match &spell.icon {
                    Some(handle) => Image::<Handle>::new(handle).width(40).height(40).into(),
                    None => vertical_space().width(40).into(),
                };

                let desc = match spell.desc.as_ref() {
                    Some(desc) => desc.data.as_str(),
                    None => "",
                };

                let elem: Element<'_> = row![icon, text(&spell.name.data).width(120), text(desc)]
                    .width(Length::Fill)
                    .spacing(16)
                    .padding(16)
                    .into();

                Some(elem)
            })
            .enumerate()
            .map(|(i, e)| hoverable(e, i, &self.hoverable_state, Message::HoverableEvent).into());

        let spells =
            Column::from_iter(spells.intersperse_with(|| horizontal_rule(1).into()));

        bordered_container(column![class_name, horizontal_rule(4), spells].width(Length::Fill))
            .width(Length::Fill)
            .into()
    }

    pub fn view<'a>(&'a self, player: &'a Player, spell_record: &'a SpellRecord) -> Element<'a> {
        let caster_classes = player.classes.iter().filter(|c| c.is_caster);

        let classes = caster_classes.map(|class| self.view_class(class, spell_record));
        let classes = classes.intersperse_with(|| vertical_space().height(32).into());

        scrollable(Column::from_iter(classes).padding(32))
            .width(Length::Fill)
            .into()
    }
}

fn bordered_container<'a>(content: impl Into<Element<'a>>) -> iced::widget::Container<'a, Message> {
    fn style(theme: &iced::Theme) -> container::Style {
        let palette = theme.palette();

        iced::widget::container::Style {
            border: iced::Border {
                width: 1.0,
                color: palette.text,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    container(content).style(style)
}
