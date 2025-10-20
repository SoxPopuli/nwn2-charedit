pub mod character;
pub mod save_file;
pub mod select_file;
pub mod settings;

use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use crate::error::Error;

pub use self::{
    character::{Message as CharacterMessage, State as CharacterState},
    save_file::{Message as SaveMessage, State as SaveState},
    select_file::{Message as SelectFileMessage, State as SelectFileState},
    settings::{Message as SettingsMessage, State as SettingsState},
};

use iced::{Element, Length, widget::container};
use regex::Regex;

pub fn bordered<'a, Msg>(view: impl Into<Element<'a, Msg>>) -> iced::widget::Container<'a, Msg>
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

    container(outer)
}

pub fn bordered_padded<'a, Msg>(
    view: impl Into<Element<'a, Msg>>,
) -> iced::widget::Container<'a, Msg>
where
    Msg: 'a,
{
    bordered(view).padding(24)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Date {
    pub day: u32,
    pub month: u32,
    pub year: u32,
    pub hour: u32,
    pub minute: u32,
}
impl Date {
    pub fn from_strings(
        day: &str,
        month: &str,
        year: &str,
        hour: &str,
        minute: &str,
    ) -> Result<Self, Error> {
        let to_parse_error = |e: std::num::ParseIntError| Error::ParseError(e.to_string());

        Ok(Self {
            day: day.parse().map_err(to_parse_error)?,
            month: month.parse().map_err(to_parse_error)?,
            year: year.parse().map_err(to_parse_error)?,
            hour: hour.parse().map_err(to_parse_error)?,
            minute: minute.parse().map_err(to_parse_error)?,
        })
    }

    pub fn hyphenated_string(&self) -> String {
        format!(
            "{}-{:02}-{:02}-{:02}-{:02}",
            &self.year, &self.month, &self.day, &self.hour, &self.minute
        )
    }

    pub fn date_string(&self) -> String {
        format!(
            "{}{:02}{:02}{:02}{:02}",
            &self.year, &self.month, &self.day, &self.hour, &self.minute
        )
    }

    pub fn pretty_string(&self) -> String {
        format!(
            "{}-{:02}-{:02} {:02}:{:02}",
            &self.year, &self.month, &self.day, &self.hour, &self.minute
        )
    }
}
impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Date {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let date: u64 = self.date_string().parse().unwrap();
        let other_date: u64 = other.date_string().parse().unwrap();

        date.cmp(&other_date)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveEntry {
    pub path: PathBuf,
    pub date: Date,
    pub number: u32,
    pub name: String,
    pub image: iced::widget::image::Handle,
}
impl SaveEntry {
    pub fn new(
        path: impl Into<PathBuf>,
        number: u32,
        date: Date,
        name: String,
        image: Vec<u8>,
    ) -> Result<Self, Error> {
        let reader = std::io::BufReader::new(std::io::Cursor::new(image));

        let image =
            image::load(reader, image::ImageFormat::Tga).expect("Failed to load save image");
        let pixels = image.to_rgba8();

        let image = iced::widget::image::Handle::from_rgba(
            pixels.width(),
            pixels.height(),
            pixels.into_vec(),
        );

        Ok(Self {
            path: path.into(),
            date,
            number,
            name,
            image,
        })
    }
}
impl PartialOrd for SaveEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SaveEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.date.cmp(&other.date)
    }
}

// 000003 - 06-10-2025-17-49
static SAVE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    let pattern = r"(?m)^0+(\d+) - (\d+)-(\d+)-(\d+)-(\d+)-(\d+)$";
    Regex::new(pattern).expect("Failed to create regex")
});

pub fn get_save_folder_name(path: impl AsRef<str>) -> Option<(u32, Date)> {
    let folder_name = path.as_ref();

    let (_, [save_no, day, month, year, hour, minute]) =
        SAVE_REGEX.captures(folder_name)?.extract();

    let date =
        Date::from_strings(day, month, year, hour, minute).expect("Failed to parse save date");
    let save_no = save_no.parse().expect("Failed to parse save number");

    Some((save_no, date))
}

pub fn get_save_folders(save_dir: &Path) -> Result<Vec<SaveEntry>, Error> {
    let entries = save_dir
        .read_dir()?
        .filter_map(|d| {
            let d = d.ok()?;
            if let Ok(m) = d.metadata()
                && m.is_dir()
            {
                let file_name = d.file_name();
                let file_name = file_name.to_str()?;

                let (save_no, date) = get_save_folder_name(file_name)?;

                let name = std::fs::read_to_string(d.path().join("savename.txt"))
                    .expect("Failed to read savename.txt");

                let image =
                    std::fs::read(d.path().join("screen.tga")).expect("Failed to read screen.tga");

                Some(
                    SaveEntry::new(d.path(), save_no, date, name, image)
                        .expect("Invalid save entry"),
                )
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(entries)
}
