#![allow(unstable_name_collisions)]

use crate::{
    ids::spell::Spell as SpellId,
    player::{Player, PlayerClass, player_class::SpellKnownList},
    spell::{Spell, SpellRecord},
    ui::{HoverableEvent, HoverableState, hoverable, search_window},
};
use iced::{
    Length,
    widget::{
        Column, Image, button, column, combo_box, container, horizontal_rule, image::Handle, row,
        scrollable, text, vertical_space,
    },
};
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    HoverableEvent(HoverableEvent),
    ClassSelected(usize),
    SpellTabSelected(usize),
    AddPressed,
    SwapPressed(usize),
    RemovePressed(usize),
    SearchWindow(search_window::Message),
}

pub type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Default, Clone)]
struct ClassOption {
    index: usize,
    name: String,
}
impl ClassOption {
    pub fn get<'a>(&self, player: &'a Player) -> Option<&'a PlayerClass> {
        player.classes.get(self.index)
    }

    pub fn get_mut<'a>(&self, player: &'a mut Player) -> Option<&'a mut PlayerClass> {
        player.classes.get_mut(self.index)
    }
}
impl std::fmt::Display for ClassOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Default)]
pub struct State {
    class_options: combo_box::State<ClassOption>,
    selected_class: ClassOption,
    hoverable_state: HoverableState,
    spell_tab: usize,
    search_window: search_window::State,
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
            search_window: Default::default(),
        }
    }

    fn get_current_spell_list<'a>(&self, player: &'a mut Player) -> Option<&'a mut SpellKnownList> {
        let class = self.selected_class.get_mut(player);

        class.and_then(
            |class| match class.spell_known_list.get_mut(self.spell_tab) {
                Some(Some(lst)) => Some(lst),
                _ => None,
            },
        )
    }

    pub fn update(&mut self, player: &mut Player, msg: Message) {
        match msg {
            Message::HoverableEvent(e) => e.update(&mut self.hoverable_state),
            Message::ClassSelected(i) => {
                self.selected_class = self.class_options.options()[i].clone();
            }
            Message::SpellTabSelected(i) => {
                self.spell_tab = i;
                self.hoverable_state.reset();
            }
            Message::AddPressed => {
                self.search_window.open(search_window::SearchMode::Add);
            }
            Message::SwapPressed(i) => {
                self.search_window.open(search_window::SearchMode::Swap(i));
            }
            Message::RemovePressed(i) => {
                self.hoverable_state.reset();
                if let Some(lst) = self.get_current_spell_list(player) {
                    lst.remove_spell(i);
                }
            }
            Message::SearchWindow(msg @ search_window::Message::Confirm) => {
                let spell_list = self.get_current_spell_list(player);

                match self.search_window.mode {
                    search_window::SearchMode::None => {}
                    search_window::SearchMode::Add => {
                        if let Some(new_id) = self.search_window.selected_id
                            && let Some(spell_list) = spell_list
                        {
                            spell_list.add_spell(SpellId(new_id.try_into().unwrap()));
                        }
                    }
                    search_window::SearchMode::Swap(index) => {
                        if let Some(new_id) = self.search_window.selected_id
                            && let Some(spell_list) = self.get_current_spell_list(player)
                        {
                            let spell = SpellId(new_id.try_into().unwrap());
                            spell_list.swap_spell(index, spell);
                        }
                    }
                }

                self.search_window.update(msg);
            }
            Message::SearchWindow(msg) => {
                self.search_window.update(msg);
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
                        let spell = spell_record.spells.get(&(x.0 as usize))?;
                        self.view_spell(spell)
                    })
                    .enumerate()
                    .map(|(i, x)| {
                        hoverable(x, i, self.hoverable_state, Message::HoverableEvent).into()
                    })
                    .intersperse_with(|| horizontal_rule(1).into());

                let col = Column::from_iter(spells)
                    // .height(Length::Shrink)
                    .width(Length::Fill);
                let col = scrollable(col).height(Length::Fill);

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

        bordered_container(spells)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn button_bar(&self) -> Element<'_> {
        let btn = |content| button(text(content).center()).width(Length::Fill);

        row![
            btn("Add").on_press(Message::AddPressed),
            btn("Swap").on_press_maybe(
                self.hoverable_state
                    .selected_entry
                    .map(Message::SwapPressed)
            ),
            btn("Remove").on_press_maybe(
                self.hoverable_state
                    .selected_entry
                    .map(Message::RemovePressed)
            )
        ]
        .spacing(8)
        .padding(8)
        .height(Length::Shrink)
        .into()
    }

    pub fn view<'a>(&'a self, player: &'a Player, spell_record: &'a SpellRecord) -> Element<'a> {
        if self.search_window.is_active() {
            let selected_class = &player.classes[self.selected_class.index];
            let class = *selected_class.class.get();
            let level = self.spell_tab;

            self.search_window
                .view(search_window::SearchKind::Spells {
                    spell_record,
                    class,
                    level: level as u8,
                })
                .map(Message::SearchWindow)
        } else {
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
                .map(|elem| container(elem).padding(16).height(Length::Fill));

            let items = column![combo,]
                .push_maybe(class)
                .push(self.button_bar())
                .padding(8.0);

            items.into()
        }
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
