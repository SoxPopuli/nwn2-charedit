use chrono::{Datelike, Timelike};
use iced::widget::{row, text_input};

use crate::{SaveFile, ui::get_save_folder_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {}

type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Default, Clone)]
pub struct State {
    pub active: bool,
    pub save_folder_name: String,
}
impl State {
    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn open(&mut self, save_file: &SaveFile) {
        self.active = true;

        let save_dir = save_file
            .path
            .parent()
            .and_then(|x| x.parent())
            .expect("Failed to get save file dir");

        let next_number = save_dir
            .read_dir()
            .into_iter()
            .flat_map(|x| {
                x.filter_map(|d| {
                    let dir = d.ok()?;
                    let file_name = dir.file_name();
                    let file_name = file_name.to_str()?;
                    get_save_folder_name(file_name)
                })
            })
            .map(|x| x.0)
            .max()
            .map(|x| x + 1)
            .unwrap_or(0);

        let now = chrono::Local::now();
        let now = super::Date {
            day: now.day(),
            month: now.month(),
            year: now.year() as u32,
            hour: now.hour(),
            minute: now.minute(),
        };

        self.save_folder_name = format!("{:06} - {}", next_number, now.hyphenated_string());
    }

    pub fn update(&mut self, msg: Message) {}

    pub fn view(&self) -> Element<'_> {
        let body = row![text_input("Save Folder Name", &self.save_folder_name),];

        super::bordered_padded(body).into()
    }
}
