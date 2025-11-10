#![allow(unstable_name_collisions)]

use crate::{
    feat::{Feat, FeatRecord},
    player::Player,
    ui::{HoverableEvent, HoverableState, hoverable},
};
use iced::{
    Length,
    widget::{Column, Image, container, horizontal_rule, horizontal_space, row, scrollable, text},
};
use itertools::Itertools;

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
    hoverable_state: HoverableState,
}
impl State {
    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::HoverableEvent(e) => e.update(&mut self.hoverable_state),
        }
    }

    fn view_feat<'a>(&'a self, index: usize, feat: &'a Feat) -> Element<'a> {
        let icon: Element<'_> = match &feat.icon {
            Some(icon) => Image::new(icon).into(),
            None => horizontal_space().width(40).into(),
        };

        let desc = feat
            .desc
            .as_ref()
            .map(|x| x.data.as_str())
            .unwrap_or_default();

        let item = row![icon, text(&feat.name.data).width(120), text(desc),]
            .width(Length::Fill)
            .padding(16)
            .spacing(16);

        hoverable(item, index, &self.hoverable_state, Message::HoverableEvent)
            .width(Length::Fill)
            .into()
    }

    pub fn view<'a>(&'a self, player: &'a Player, feat_record: &'a FeatRecord) -> Element<'a> {
        let feats = {
            let feats = player.feats.list_ref.get();
            let feats = feats
                .iter()
                .map(|x| x.get())
                .filter_map(|x| {
                    let id: usize = (*x).into();
                    feat_record.feats.get(&id)
                })
                .enumerate()
                .map(|(i, feat)| self.view_feat(i, feat))
                .intersperse_with(|| horizontal_rule(1).into());
            bordered_container(Column::from_iter(feats))
        };

        scrollable(container(feats).padding(32)).into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    HoverableEvent(HoverableEvent),
}

pub type Element<'a> = iced::Element<'a, Message>;
