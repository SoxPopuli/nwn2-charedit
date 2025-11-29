#![allow(unstable_name_collisions)]

use crate::{
    feat::{Feat, FeatRecord},
    player::Player,
    ui::{HoverableEvent, HoverableState, hoverable, search_window},
};
use iced::{
    Length,
    widget::{
        Column, Image, button, column, container, horizontal_rule, horizontal_space, row,
        scrollable, text,
    },
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

#[derive(Default)]
pub struct State {
    hoverable_state: HoverableState,
    search_window: search_window::State,
}
impl State {
    pub fn update(&mut self, player: &mut Player, msg: Message) {
        match msg {
            Message::HoverableEvent(e) => e.update(&mut self.hoverable_state),
            Message::AddPressed => {
                self.search_window.open(search_window::SearchMode::Add);
            }
            Message::SwapPressed(idx) => {
                self.search_window
                    .open(search_window::SearchMode::Swap(idx));
            }
            Message::RemovePressed(idx) => {
                self.hoverable_state.reset();
                player.feats.remove_feat(idx);
            }
            Message::SearchWindow(msg @ search_window::Message::Confirm) => {
                match self.search_window.mode {
                    search_window::SearchMode::None => {}
                    search_window::SearchMode::Add => {
                        if let Some(new_id) = self.search_window.selected_id {
                            player.feats.add_feat(new_id.try_into().unwrap());
                        }
                    }
                    search_window::SearchMode::Swap(old_index) => {
                        if let Some(new_id) = self.search_window.selected_id {
                            player
                                .feats
                                .swap_feat(old_index, new_id.try_into().unwrap());
                        }
                    }
                }

                self.search_window.update(msg);
            }
            Message::SearchWindow(msg) => self.search_window.update(msg),
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

        hoverable(item, index, self.hoverable_state, Message::HoverableEvent)
            .width(Length::Fill)
            .into()
    }

    fn button_bar<'a>(&self) -> Element<'a> {
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

    fn view_feats<'a>(
        &'a self,
        player: &'a Player,
        feat_record: &'a FeatRecord,
    ) -> impl Into<Element<'a>> {
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
        let feats = scrollable(container(feats).padding(32)).height(Length::Fill);

        column![feats, self.button_bar()].padding(8.0)
    }

    pub fn view<'a>(&'a self, player: &'a Player, feat_record: &'a FeatRecord) -> Element<'a> {
        if self.search_window.is_active() {
            self.search_window
                .view(search_window::SearchKind::Feats(feat_record))
                .map(Message::SearchWindow)
        } else {
            self.view_feats(player, feat_record).into()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    HoverableEvent(HoverableEvent),
    AddPressed,
    SwapPressed(usize),
    RemovePressed(usize),
    SearchWindow(search_window::Message),
}

pub type Element<'a> = iced::Element<'a, Message>;
