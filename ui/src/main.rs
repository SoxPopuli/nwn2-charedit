mod error;
mod feat;
mod field_ref;
mod ids;
mod player;
mod spell;
mod tlk_string_ref;
mod two_d_array;
mod ui;

use crate::{
    error::Error, player::Player, two_d_array::FileReader2DA, ui::settings::GameResources,
};
use iced::{
    Length, Task,
    widget::{button, column, horizontal_space, row, text},
};
use nwn_lib::files::gff::Gff;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

pub(crate) fn join_path(base: &Path, paths: &[&str]) -> PathBuf {
    let paths = paths.join(std::path::MAIN_SEPARATOR_STR);
    base.join(paths)
}

fn open_file(path: &Path) -> Result<Gff, Error> {
    let ext = path.extension().and_then(|x| x.to_str());

    match ext {
        Some("zip") => {
            let file = File::open(path).unwrap();
            let mut reader = zip::read::ZipArchive::new(file).unwrap();
            let save = {
                let mut save = reader
                    .by_name("playerlist.ifo")
                    .expect("missing playerlist.ifo");
                let mut buf = Vec::with_capacity(save.size() as usize);
                save.read_to_end(&mut buf).unwrap();
                std::io::Cursor::new(buf)
            };

            Gff::read_without_tlk(save).map_err(|e| e.into())
        }
        Some("ifo") => {
            let file = File::open(path).unwrap();
            Gff::read_without_tlk(file).map_err(|e| e.into())
        }

        Some(e) => panic!("unexpected file ext: {e}"),
        None => panic!("unknown file type"),
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Message {
    FileSelected(PathBuf),
    Settings(ui::SettingsMessage),
    Character(ui::CharacterMessage),
    OpenSettings,
    OpenFileSelector,
    FileSelector(ui::SelectFileMessage),
    SaveFile,
    SaveWindow(ui::SaveMessage),
    CloseFile,
}

type Element<'a> = iced::Element<'a, Message>;

fn menu_button(text: &str) -> iced::widget::Button<'_, Message> {
    let style = |theme: &iced::Theme, status| {
        use iced::{
            Background, Border, Color,
            widget::button::{Status, Style},
        };

        let palette = theme.palette();

        let background = match status {
            Status::Hovered => Some(Background::Color(Color {
                a: 0.25,
                ..palette.text
            })),
            _ => None,
        };

        Style {
            text_color: match status {
                Status::Disabled => palette.text.scale_alpha(0.5),
                _ => palette.text,
            },
            background,
            border: Border::default().rounded(8.0),
            ..Default::default()
        }
    };

    button(text).style(style)
}

pub type Tlk = nwn_lib::files::tlk::Tlk<BufReader<File>>;

#[derive(Debug)]
pub struct SaveFile {
    pub file: Gff,
    pub path: PathBuf,
}
impl SaveFile {
    pub fn get_players(&self, tlk: &Tlk, reader_2da: &mut FileReader2DA) -> Vec<Player> {
        let player_list = self
            .file
            .root
            .bfs_iter()
            .find(|x| x.has_label("Mod_PlayerList"))
            .expect("Couldn't find player list");

        let lock = player_list.read().unwrap();
        let player_list = lock.field.expect_list().unwrap();

        player_list
            .iter()
            .map(|x| Player::new(tlk, reader_2da, x))
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    pub fn save_changes<W>(&mut self, output: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        Ok(self.file.write(output)?)
    }
}

pub fn show_error_popup(msg: impl Into<String>) {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("Error")
        .set_description(msg)
        .show();
}

fn show_error_popup_task(msg: impl Into<String>) -> iced::Task<Message> {
    show_error_popup(msg);
    Task::none()
}

/// Show error popup then panic
#[macro_export]
macro_rules! popup_panic {
    ($msg:tt) => {{
        $crate::show_error_popup(format!($msg));
        panic!($msg);
    }};
}

/// Show error popup then return `None`
#[macro_export]
macro_rules! popup_opt {
    ($msg:tt) => {{
        $crate::show_error_popup(format!($msg));
        None
    }};
}

#[derive(Debug, Default)]
struct App {
    pub save_file: Option<SaveFile>,
    pub characters: ui::CharacterState,
    pub settings: ui::SettingsState,
    pub select_file: ui::SelectFileState,
    pub save_window: ui::SaveState,
}
impl App {
    fn title() -> &'static str {
        env!("CARGO_BIN_NAME")
    }

    fn close_windows(&mut self) {
        self.settings.close();
        self.select_file.close();
        self.save_window.close();
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    fn init() -> (Self, Task<Message>) {
        let this = App {
            save_file: None,
            characters: Default::default(),
            settings: ui::SettingsState::from_file_or_default(),
            select_file: ui::SelectFileState::default(),
            save_window: ui::SaveState::default(),
        };

        (this, Task::none())
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::FileSelected(path) => match open_file(&path) {
                Ok(save) => {
                    match self.settings.game_resources.as_mut() {
                        Some(g) => {
                            let save_file = SaveFile { file: save, path };

                            self.characters = ui::character::State::new(
                                save_file.get_players(&g.tlk, &mut g.file_reader),
                            );
                            self.save_file = Some(save_file);
                        }
                        None => {
                            return show_error_popup_task(
                                "Couldn't find game resources, is Game Directory set?".to_string(),
                            );
                        }
                    };
                }
                Err(e) => show_error_popup(format!("Failed to open save file: {e}")),
            },
            Message::Settings(m @ ui::SettingsMessage::Save) => {
                self.settings.update(m);
            }
            Message::Settings(m) => {
                self.settings.update(m);
            }
            Message::OpenSettings => {
                self.close_windows();
                self.settings.active = true;
            }
            Message::OpenFileSelector => {
                if self.settings.save_dir.is_some() {
                    self.close_windows();
                }
                if let Some(dir) = &self.settings.save_dir {
                    self.select_file.open(dir);
                } else {
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Info)
                        .set_description("Save directory not set")
                        .show();
                }
            }
            Message::FileSelector(m) => {
                return self.select_file.update(m);
            }
            Message::Character(msg) => {
                self.characters.update(msg);
            }
            Message::SaveFile => {
                if self.save_file.is_some() {
                    self.close_windows();
                }
                if let Some(save) = &self.save_file {
                    self.save_window.open(save);
                }
            }
            Message::SaveWindow(msg) => self.save_window.update(msg),
            Message::CloseFile => {
                let settings = std::mem::take(&mut self.settings);

                *self = App {
                    settings,
                    ..Default::default()
                }
            }
        }

        Task::none()
    }

    fn menu(&self) -> Element<'_> {
        let open_file = menu_button("Open").on_press(Message::OpenFileSelector);
        let save =
            menu_button("Save").on_press_maybe(self.save_file.as_ref().map(|_| Message::SaveFile));
        let settings = menu_button("Settings").on_press(Message::OpenSettings);

        let mut menu_bar = row![open_file, save, settings].spacing(8);

        if self.save_file.is_some() {
            menu_bar = menu_bar
                .push(horizontal_space().width(Length::Fill))
                .push(menu_button("Close").on_press(Message::CloseFile));
        }

        column![menu_bar, iced::widget::horizontal_rule(4)]
            .spacing(4)
            .padding(iced::Padding {
                top: 4.0,
                left: 2.0,
                bottom: 8.0,
                ..Default::default()
            })
            .into()
    }

    fn view(&self) -> Element<'_> {
        let body = if self.settings.active {
            self.settings.view().map(Message::Settings)
        } else if self.save_window.active {
            self.save_window.view().map(Message::SaveWindow)
        } else if self.select_file.active {
            self.select_file.view().map(Message::FileSelector)
        } else {
            match &self.settings.game_resources {
                Some(GameResources {
                    spell_record,
                    feat_record,
                    ..
                }) => self
                    .characters
                    .view(spell_record, feat_record)
                    .map(Message::Character),
                None => text("Game Directory not set correctly").into(),
            }
        };

        column![self.menu(), body].into()
    }

    fn run() -> Result<(), iced::Error> {
        iced::application(Self::title(), Self::update, Self::view)
            .centered()
            .window_size((640.0, 480.0))
            .theme(Self::theme)
            .run_with(Self::init)
    }
}

fn main() {
    App::run().unwrap()
}

#[cfg(test)]
mod tests {}
