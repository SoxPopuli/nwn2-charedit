use crate::error::Error;
use iced::{
    Length, Task,
    widget::{Column, button, column, container, horizontal_space, row, text, vertical_space},
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Close,
    Open(usize),
    MouseEntered(usize),
    MouseExited(usize),
    EntrySelected(usize),
}

type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Date {
    pub day: u32,
    pub month: u32,
    pub year: u32,
    pub hour: u32,
    pub minute: u32,
}
impl Date {
    pub fn new(
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
        number: &str,
        date: Date,
        name: String,
        image: Vec<u8>,
    ) -> Result<Self, Error> {
        let to_parse_error = |e: std::num::ParseIntError| Error::ParseError(e.to_string());

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
            number: number.parse().map_err(to_parse_error)?,
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

#[derive(Debug, Default)]
pub struct State {
    pub active: bool,
    hovered_entry: Option<usize>,
    selected_entry: Option<usize>,
    save_entries: Vec<SaveEntry>,
}
impl State {
    pub fn open(&mut self, save_dir: &Path) {
        if let Ok(mut entries) = Self::get_save_folders(save_dir) {
            entries.sort_by(|a, b| b.cmp(a));
            self.save_entries = entries;
        }

        self.active = true;
    }

    pub fn close(&mut self) {
        self.active = false;
        self.selected_entry = None;
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<crate::Message> {
        match msg {
            Message::Close => self.close(),
            Message::MouseEntered(idx) => {
                self.hovered_entry = Some(idx);
            }
            Message::MouseExited(idx) => {
                if Some(idx) == self.hovered_entry {
                    self.hovered_entry = None;
                }
            }
            Message::EntrySelected(idx) => {
                self.selected_entry = Some(idx);
            }
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

    fn get_save_folders(save_dir: &Path) -> Result<Vec<SaveEntry>, Error> {
        // 000003 - 06-10-2025-17-49
        let re = {
            let pattern = r"(?m)^0+(\d+) - (\d+)-(\d+)-(\d+)-(\d+)-(\d+)$";
            std::sync::LazyLock::new(|| regex::Regex::new(pattern).expect("Failed to create regex"))
        };

        let entries = save_dir
            .read_dir()?
            .filter_map(|d| {
                let d = d.ok()?;
                if let Ok(m) = d.metadata()
                    && m.is_dir()
                {
                    let file_name = d.file_name();
                    let file_name = file_name.to_str()?;

                    let (_, [save_no, day, month, year, hour, minute]) =
                        re.captures(file_name)?.extract();

                    let name = std::fs::read_to_string(d.path().join("savename.txt"))
                        .expect("Failed to read savename.txt");

                    let image = std::fs::read(d.path().join("screen.tga"))
                        .expect("Failed to read screen.tga");

                    let date = Date::new(day, month, year, hour, minute).unwrap();

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
        let items = iced::widget::mouse_area(items)
            .on_enter(Message::MouseEntered(index))
            .on_exit(Message::MouseExited(index))
            .on_press(Message::EntrySelected(index));

        container(items)
            .style(move |theme: &iced::Theme| {
                let p = theme.extended_palette();
                container::Style {
                    background: if self.selected_entry == Some(index) {
                        Some(iced::Background::Color(p.primary.strong.color))
                    } else if self.hovered_entry == Some(index) {
                        Some(iced::Background::Color(p.background.strong.color))
                    } else {
                        None
                    },
                    ..Default::default()
                }
            })
            .into()
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
                button("Open").on_press_maybe(match self.selected_entry {
                    Some(i) => Some(Message::Open(i)),
                    None => None,
                }),
            ]
            .height(Length::Fixed(32.0))
            .spacing(16),
        ];

        super::bordered(body.into())
    }
}
