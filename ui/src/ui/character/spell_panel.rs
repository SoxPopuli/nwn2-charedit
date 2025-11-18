#![allow(unstable_name_collisions)]

use crate::{
    player::{Player, PlayerClass},
    spell::{Spell, SpellRecord},
    ui::{HoverableEvent, HoverableState, hoverable},
};
use iced::{
    Length,
    widget::{
        Column, Image, column, combo_box, container, horizontal_rule, image::Handle, row,
        scrollable, text, vertical_space,
    },
};
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    HoverableEvent(HoverableEvent),
    ClassSelected(usize),
    SpellTabSelected(usize),
}

pub type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Default, Clone)]
struct ClassOption {
    index: usize,
    name: String,
}
impl std::fmt::Display for ClassOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, Default, Clone)]
pub struct State {
    class_options: combo_box::State<ClassOption>,
    selected_class: ClassOption,
    hoverable_state: HoverableState,
    spell_tab: usize,
}
impl State {
    pub fn new(player: &Player) -> Self {
        let class_options = combo_box::State::new(
            player
                .classes
                .iter()
                .enumerate()
                .filter(|(_, class)| class.is_caster)
                .map(|(i, class)| {
                    let name = class.class.get().to_string();
                    ClassOption { index: i, name }
                })
                .collect(),
        );

        let selected_class = class_options
            .options()
            .first()
            .cloned()
            .expect("No caster classes found");

        Self {
            class_options,
            selected_class,
            hoverable_state: Default::default(),
            spell_tab: 0,
        }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::HoverableEvent(e) => e.update(&mut self.hoverable_state),
            Message::ClassSelected(i) => {
                self.selected_class = self.class_options.options()[i].clone();
            }
            Message::SpellTabSelected(i) => {
                self.spell_tab = i;
                self.hoverable_state.reset();
            }
        }
    }

    fn view_spell<'a>(&self, spell: &'a Spell) -> Option<Element<'a>> {
        let icon: Element<'_> = match &spell.icon {
            Some(handle) => Image::<Handle>::new(handle).width(40).height(40).into(),
            None => vertical_space().width(40).into(),
        };

        let name = spell.name.data.trim();

        let desc = match spell.desc.as_ref() {
            Some(desc) => desc.data.as_str().trim(),
            None => "",
        };

        let elem: Element<'_> = row![icon, text(name).width(120), text(desc)]
            .width(Length::Fill)
            .spacing(16)
            .padding(16)
            .into();

        Some(elem)
    }

    fn view_spells<'a>(
        &self,
        class: &'a PlayerClass,
        spell_record: &'a SpellRecord,
    ) -> Element<'a> {
        let spells = &class.spell_known_list;

        let tabs = spells.iter().map_while(|x| x.as_ref()).enumerate().fold(
            iced_aw::Tabs::new(Message::SpellTabSelected),
            |tabs, (i, spells)| {
                let spells = spells
                    .spells
                    .iter()
                    .filter_map(|x| {
                        let spell = spell_record.spells.get(&x.0)?;
                        self.view_spell(spell)
                    })
                    .enumerate()
                    .map(|(i, x)| {
                        hoverable(x, i, self.hoverable_state, Message::HoverableEvent).into()
                    })
                    .intersperse_with(|| horizontal_rule(1).into());

                let col = Column::from_iter(spells)
                    .width(Length::Fill)
                    .height(Length::Shrink);
                let col = scrollable(col);

                tabs.push(i, iced_aw::TabLabel::Text(i.to_string()), col)
            },
        );

        tabs.set_active_tab(&self.spell_tab).into()
    }

    fn view_class<'a>(
        &'a self,
        class: &'a PlayerClass,
        spell_record: &'a SpellRecord,
    ) -> Element<'a> {
        let spells = self.view_spells(class, spell_record);

        bordered_container(spells).width(Length::Fill).into()
    }

    pub fn view<'a>(&'a self, player: &'a Player, spell_record: &'a SpellRecord) -> Element<'a> {
        let mut caster_classes = player.classes.iter().filter(|c| c.is_caster);

        let combo = iced::widget::combo_box(
            &self.class_options,
            "Select class",
            Some(&self.selected_class),
            |item| Message::ClassSelected(item.index),
        );

        let class = caster_classes
            .nth(self.selected_class.index)
            .map(|c| self.view_class(c, spell_record))
            .map(|elem| container(elem).padding(16));

        let items = column![
            combo,
            // scrollable(Column::from_iter(classes).padding(32)).width(Length::Fill)
        ]
        .push_maybe(class)
        .padding(8.0);

        items.into()
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
