pub mod select_file;
pub mod settings;

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
