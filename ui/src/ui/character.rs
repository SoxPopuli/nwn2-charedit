#![allow(unstable_name_collisions)]

mod feat_panel;

use iced::{
    Length,
    widget::{
        Column, column, container, horizontal_rule,
        image::{Handle, Image},
        row, scrollable, text, vertical_space,
    },
};
use iced_aw::{TabLabel, grid, grid_row, tabs::Tabs};
use itertools::Itertools;
use nwn_lib::files::gff::field::Field;

use crate::{
    feat::FeatRecord,
    field_ref::FieldRef,
    player::{Player, PlayerClass},
    spell::SpellRecord,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    TabSelected(TabMode),
    StatChanged { stat: Stat, new_value: u8 },
    FeatPanel(feat_panel::Message),
}

type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TabMode {
    #[default]
    Stats,
    Spells,
    Feats,
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

#[derive(Debug, Default, Clone)]
pub struct State {
    pub selected_player: usize,
    pub players: Vec<Player>,
    pub tab_mode: TabMode,

    feat_panel: feat_panel::State,
}
impl State {
    pub fn new(players: Vec<Player>) -> Self {
        Self {
            tab_mode: TabMode::Stats,
            selected_player: 0,
            players,
            feat_panel: Default::default(),
        }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::TabSelected(mode) => {
                self.tab_mode = mode;
            }
            Message::StatChanged { stat, new_value } => {
                let player = self.players.get_mut(self.selected_player);
                let player = match player {
                    Some(player) => player,
                    None => return,
                };

                let set_stat = |field_ref: &mut FieldRef<u8>| {
                    field_ref.set(new_value, |x| Field::Byte(*x));
                };

                match stat {
                    Stat::Strength => set_stat(&mut player.attributes.str),
                    Stat::Dexterity => set_stat(&mut player.attributes.dex),
                    Stat::Constitution => set_stat(&mut player.attributes.con),
                    Stat::Intelligence => set_stat(&mut player.attributes.int),
                    Stat::Wisdom => set_stat(&mut player.attributes.wis),
                    Stat::Charisma => set_stat(&mut player.attributes.cha),
                }
            }
            Message::FeatPanel(m) => self.feat_panel.update(m),
        }
    }

    fn view_stats(&self, player: &Player) -> Element<'_> {
        let level = player
            .classes
            .iter()
            .fold(0, |acc, class| acc + class.level.get());

        let classes = player
            .classes
            .iter()
            .map(|c| format!("{} ({})", c.class.get(), c.level.get()))
            .reduce(|acc, x| format!("{acc} | {x}"))
            .unwrap_or("None".to_string());

        let race = player.race.to_string();
        let name = format!("{} {}", player.first_name.get(), player.last_name.get());

        let stat_row = |name, value, stat| {
            let input = iced_aw::number_input(value, ..=u8::MAX, move |x| Message::StatChanged {
                stat,
                new_value: x,
            })
            .ignore_buttons(true);

            grid_row![text(name), input]
        };

        let strength = player.attributes.str.get();
        let dexterity = player.attributes.dex.get();
        let constitution = player.attributes.con.get();
        let wisdom = player.attributes.wis.get();
        let intelligence = player.attributes.int.get();
        let charisma = player.attributes.cha.get();

        let stat_grid = grid![
            stat_row("Strength", strength, Stat::Strength),
            stat_row("Dexterity", dexterity, Stat::Dexterity),
            stat_row("Constitution", constitution, Stat::Constitution),
            stat_row("Intelligence", intelligence, Stat::Intelligence),
            stat_row("Wisdom", wisdom, Stat::Wisdom),
            stat_row("Charisma", charisma, Stat::Charisma),
        ]
        .column_spacing(16);

        column![
            text(name),
            text(format!("Level {level} {race}")),
            text(classes),
            vertical_space().height(32),
            stat_grid
        ]
        .padding(16)
        .into()
    }

    fn view_spells<'a>(&'a self, player: &'a Player, spell_record: &'a SpellRecord) -> Element<'a> {
        let caster_classes = player.classes.iter().filter(|c| c.is_caster);

        fn view_class<'a>(class: &'a PlayerClass, spell_record: &'a SpellRecord) -> Element<'a> {
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

                    let elem: Element<'_> =
                        row![icon, text(&spell.name.data).width(120), text(desc)]
                            .width(Length::Fill)
                            .spacing(16)
                            .padding(16)
                            .into();
                    Some(elem)
                });

            let spells = Column::from_iter(spells.intersperse_with(|| horizontal_rule(2).into()))
                .spacing(16);

            bordered_container(column![class_name, horizontal_rule(4), spells].width(Length::Fill))
                .width(Length::Fill)
                .into()
        }

        let classes = caster_classes.map(|class| view_class(class, spell_record));
        let classes = classes.intersperse_with(|| vertical_space().height(32).into());

        scrollable(Column::from_iter(classes).padding(32))
            .width(Length::Fill)
            .into()
    }

    pub fn view<'a>(
        &'a self,
        spell_record: &'a SpellRecord,
        feat_record: &'a FeatRecord,
    ) -> Element<'a> {
        let player = match self.players.get(self.selected_player) {
            Some(player) => player,
            None => return iced::widget::vertical_space().into(),
        };

        let is_caster = player.classes.iter().any(|x| x.is_caster);

        let mut tabs = Tabs::new(Message::TabSelected)
            .push(
                TabMode::Stats,
                TabLabel::Text("Stats".to_string()),
                self.view_stats(player),
            )
            .push(
                TabMode::Feats,
                TabLabel::Text("Feats".to_string()),
                self.feat_panel
                    .view(player, feat_record)
                    .map(Message::FeatPanel),
            );

        if is_caster {
            tabs = tabs.push(
                TabMode::Spells,
                TabLabel::Text("Spells".to_string()),
                self.view_spells(player, spell_record),
            )
        }

        tabs.set_active_tab(&self.tab_mode).into()
    }
}
