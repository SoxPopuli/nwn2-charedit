use std::path::{Path, PathBuf};

use chrono::{Datelike, Timelike};
use iced::{
    widget::{button, column, horizontal_space, row, text_input, text, vertical_space}, Length
};

use crate::{SaveFile, error::Error, ui::get_save_folder_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    NameChanged(String),
    Save,
    Close,
}

type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug)]
enum SaveFileKind {
    Zip(PathBuf),
    Unpacked(PathBuf),
}
impl SaveFileKind {
    fn save_into(self, save_file: &SaveFile) -> Result<(), Error> {
        match self {
            Self::Zip(path) => {
                let f = std::fs::File::open(&path).map(std::io::BufReader::new)?;
                let mut archive =
                    zip::ZipArchive::new(f).map_err(|e| Error::ParseError(e.to_string()))?;
                let file_count = archive.len();

                let mut files = (0..file_count)
                    .filter_map(|i| {
                        let mut file = archive.by_index(i).ok()?;
                        let name = file.name().to_string();

                        let data = {
                            use std::io::Read;
                            let mut buf = Vec::with_capacity(file.size() as usize);
                            file.read_to_end(&mut buf).map(|_| buf)
                        }
                        .expect("Failed to read zip data");

                        Some((name, data))
                    })
                    .collect::<Vec<_>>();

                let save_data = {
                    let mut buf = std::io::BufWriter::new(Vec::new());
                    save_file.save_changes(&mut buf).and_then(|()| {
                        buf.into_inner().map_err(|e| {
                            Error::WriteError(format!("Failed to flush save buffer: {e}"))
                        })
                    })
                }?;

                let playerlist = files
                    .iter_mut()
                    .find(|x| x.0.eq_ignore_ascii_case("playerlist.ifo"))
                    .expect("Couldn't find playerlist in save files");
                playerlist.1 = save_data;

                drop(archive);

                let f = std::fs::File::create(&path)?;
                let f = std::io::BufWriter::new(f);

                let mut writer = zip::ZipWriter::new(f);

                for (name, data) in files {
                    use std::io::Write;

                    let options = zip::write::SimpleFileOptions::default();
                    writer.start_file(&name, options).map_err(|e| {
                        Error::WriteError(format!("Failed to start writing file [{name}]: {e}"))
                    })?;

                    writer.write_all(&data)?;
                }
            }
            Self::Unpacked(path) => {
                let f = std::fs::File::create(path)?;
                let mut f = std::io::BufWriter::new(f);

                save_file.save_changes(&mut f)?;
            }
        }

        Ok(())
    }

    fn from_game_dir(dir: &Path) -> Option<Self> {
        let from_entry = |entry: std::fs::DirEntry| {
            let name = entry.file_name();
            let name = name.to_str();

            match name {
                Some("resgff.zip") => Some(Self::Zip(entry.path())),
                Some("playerlist.ifo") => Some(Self::Unpacked(entry.path())),
                _ => None,
            }
        };

        match dir.read_dir() {
            Ok(mut r) => r.find_map(|x| match x {
                Ok(entry) => from_entry(entry),
                Err(_) => None,
            }),
            Err(_) => None,
        }
    }
}

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

        let next_number = save_file
            .save_dir
            .parent()
            .unwrap()
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

    fn save(&self, save_file: &SaveFile) {
        let dest_path = save_file
            .save_dir
            .parent()
            .expect("Failed to get save folder parent dir")
            .join(&self.save_folder_name);

        copy_dir(&save_file.save_dir, &dest_path).expect("Failed to copy save folder");

        let kind = SaveFileKind::from_game_dir(&dest_path).expect("Failed to find dest save file");
        kind.save_into(save_file).expect("Failed to save");

        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Info)
            .set_description(format!("Saved to {}", dest_path.display()))
            .show();
    }

    pub fn update(&mut self, msg: Message, save_file: &SaveFile) {
        match msg {
            Message::NameChanged(new_name) => {
                self.save_folder_name = new_name;
            }
            Message::Save => {
                self.save(save_file);
            }
            Message::Close => {
                self.close();
            }
        }
    }

    pub fn view(&self) -> Element<'_> {
        let body = column![
            text("Save Folder Name"),
            text_input("Save Folder Name", &self.save_folder_name).on_input(Message::NameChanged),
            vertical_space().height(Length::Fill),
            row![
                horizontal_space(),
                button("Close").on_press(Message::Close),
                button("Save").on_press(Message::Save),
            ]
            .height(Length::Fixed(32.0))
            .spacing(16)
        ];

        super::bordered_padded(body).into()
    }
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    use std::fs::*;

    create_dir_all(dst)?;

    for entry in read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            copy_dir(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }

    Ok(())
}
