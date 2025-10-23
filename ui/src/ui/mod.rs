pub mod select_file;
pub mod settings;
pub mod panels;

pub use self::{
    select_file::{Message as SelectFileMessage, State as SelectFileState},
    settings::{Message as SettingsMessage, State as SettingsState},
};

use iced::{Element, Length, widget::container};

pub fn bordered<'a, Msg>(view: Element<'a, Msg>) -> Element<'a, Msg>
where
    Msg: 'a,
{
    let inner = container(view).height(Length::Fill);

    let outer = container(inner)
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &iced::Theme| {
            let p = theme.palette();
            container::Style {
                border: iced::Border {
                    width: 2.0,
                    color: p.text,
                    ..Default::default()
                },
                ..Default::default()
            }
        });

    container(outer).padding(24).into()
}
