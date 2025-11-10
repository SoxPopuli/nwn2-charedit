use crate::ui::{HoverableEvent, HoverableState, SaveEntry};
use iced::{
    Length, Task,
    widget::{Column, button, column, horizontal_space, row, text, vertical_space},
};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Close,
    Open(usize),
    HoverableEvent(HoverableEvent),
}

type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Default)]
pub struct State {
    pub active: bool,
    hoverable_state: HoverableState,
    save_entries: Vec<SaveEntry>,
}
impl State {
    pub fn open(&mut self, save_dir: &Path) {
        if let Ok(mut entries) = super::get_save_folders(save_dir) {
            entries.sort_by(|a, b| b.cmp(a));
            self.save_entries = entries;
        }

        self.active = true;
    }

    pub fn close(&mut self) {
        self.active = false;
        self.hoverable_state.selected_entry = None;
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<crate::Message> {
        match msg {
            Message::Close => self.close(),
            Message::HoverableEvent(e) => e.update(&mut self.hoverable_state),
            Message::Open(idx) => {
                if let Some(entry) = self.save_entries.get(idx) {
                    let mut file_path = entry.path.join("resgff.zip");
                    if !file_path.exists() {
                        file_path = entry.path.join("playerlist.ifo");
                    }

                    let selected_task = Task::done(crate::Message::FileSelected(file_path));
                    let close_task = Task::done(crate::Message::FileSelector(Message::Close));
                    return selected_task.chain(close_task);
                }
            }
        };

        Task::none()
    }

    fn view_save_entry(&self, index: usize, entry: &SaveEntry) -> Element<'_> {
        let image = iced::widget::image(entry.image.clone())
            .width(iced::Length::Fixed(240.0))
            .height(iced::Length::Fixed(120.0))
            .content_fit(iced::ContentFit::Contain);
        let label = text(format!(
            "{} - {} - {}",
            entry.number,
            entry.name,
            entry.date.pretty_string()
        ));

        let items = row![image, label].width(Length::Fill).padding(8.0);

        super::hoverable(items, index, &self.hoverable_state, Message::HoverableEvent).into()
    }

    pub fn view(&self) -> Element<'_> {
        let entries = self
            .save_entries
            .iter()
            .enumerate()
            .map(|(i, x)| self.view_save_entry(i, x));

        let body = Column::from_iter(entries).spacing(16.0);
        let body = iced::widget::scrollable(body)
            .width(Length::Fill)
            .height(Length::Fill);

        let body = column![
            body,
            vertical_space().height(8),
            row![
                horizontal_space().width(Length::Fill),
                button("Close").on_press(Message::Close),
                button("Open")
                    .on_press_maybe(self.hoverable_state.selected_entry.map(Message::Open)),
            ]
            .height(Length::Fixed(32.0))
            .spacing(16),
        ];

        super::bordered_padded(body).into()
    }
}
