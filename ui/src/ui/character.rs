#![allow(unstable_name_collisions)]

use iced::{
    Length,
    widget::{
        Column, column, container, horizontal_rule, horizontal_space,
        image::{Handle, Image},
        row, scrollable, text, vertical_space,
    },
};
use iced_aw::{TabLabel, tabs::Tabs};
use itertools::Itertools;

use crate::{
    feat::{Feat, FeatRecord},
    player::{Player, PlayerClass},
    spell::SpellRecord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    TabSelected(TabMode),
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
}
impl State {
    pub fn new(players: Vec<Player>) -> Self {
        Self {
            tab_mode: TabMode::Stats,
            selected_player: 0,
            players,
        }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::TabSelected(mode) => {
                self.tab_mode = mode;
            }
        }
    }

    fn view_stats(&self, player: &Player) -> Element<'_> {
        let level = player
            .classes
            .iter()
            .fold(0, |acc, class| acc + class.level.get());

        let race = player.race.to_string();
        let name = format!("{} {}", player.first_name.get(), player.last_name.get());

        column![text(name), text(format!("Level {level} {race}"))]
            .padding(16)
            .into()
    }

    fn view_feats<'a>(&'a self, player: &'a Player, feat_record: &'a FeatRecord) -> Element<'a> {
        fn view_feat<'a>(feat: &'a Feat) -> Element<'a> {
            let icon: Element<'_> = match &feat.icon {
                Some(icon) => Image::new(icon).into(),
                None => horizontal_space().width(40).into(),
            };

            let desc = feat
                .desc
                .as_ref()
                .map(|x| x.data.as_str())
                .unwrap_or_default();

            row![icon, text(&feat.name.data).width(120), text(desc),]
                .padding(16)
                .spacing(16)
                .into()
        }

        let feats = {
            let feats = player.feats.list_ref.get();
            let feats = feats
                .iter()
                .map(|x| x.get())
                .filter_map(|x| {
                    let id: usize = (*x).into();
                    feat_record.feats.get(&id)
                })
                .map(view_feat)
                .intersperse_with(|| horizontal_rule(2).into());
            bordered_container(Column::from_iter(feats).spacing(16))
        };

        scrollable(container(feats).padding(32)).into()
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
                self.view_feats(player, feat_record),
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
