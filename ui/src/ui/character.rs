#![allow(unstable_name_collisions)]

use iced::widget::{
    Column, column, horizontal_rule, horizontal_space, image::Image, row, scrollable, text,
};
use iced_aw::{TabLabel, tabs::Tabs};
use itertools::Itertools;

use crate::{
    feat::{Feat, FeatRecord},
    player::Player,
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

        scrollable(Column::from_iter(feats).spacing(16)).into()
    }

    fn view_spells<'a>(&'a self, player: &'a Player, spell_record: &'a SpellRecord) -> Element<'a> {
        let caster_classes =
            player.classes
                .iter()
                .filter(|c| c.is_caster);

        text("spells").into()
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
