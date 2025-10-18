use crate::error::Error;
use cfg_if::cfg_if;
use iced::{
    Length,
    widget::{button, column, horizontal_space, row, text, text_input, vertical_space},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickDirMode {
    Game,
    Save,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    SetGameDir(String),
    SetSaveDir(String),
    Close,
    Save,
    PickDir(PickDirMode),
}

type Element<'a> = iced::Element<'a, Message>;

fn get_cache_dir() -> Result<PathBuf, Error> {
    fn get_var(var: &'static str) -> Result<String, Error> {
        std::env::var(var).map_err(|_| Error::EnvNotFound { var })
    }

    let base_dir = {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                get_var("LOCALAPPDATA").map(PathBuf::from)
            } else if #[cfg(target_os = "macos")] {
                get_var("HOME")
                    .map(|s| PathBuf::from(s)
                        .join("Library")
                        .join("Caches")
                )
            } else if #[cfg(target_os = "linux")] {
                    std::env::var("XDG_CACHE_HOME")
                    .map(PathBuf::from)
                    .or_else(|_|
                        Ok::<_, Error>(Path::new(&get_var("HOME")?)
                        .join(".cache"))
                    )
            } else {
                compile_error!("target os not supported")
            }
        }
    }?;

    let dir = base_dir.join("nwn2-charedit");

    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("Failed to create dir {}: {e}", dir.display()));
    }

    Ok(dir)
}

fn get_cache_file_path() -> PathBuf {
    let cache_dir = get_cache_dir().expect("Failed to get cache dir");
    cache_dir.join("settings.json")
}

#[derive(Debug, Serialize, Deserialize)]
struct SavedSettings {
    save_dir: Option<PathBuf>,
    game_dir: Option<PathBuf>,
}

fn save_settings(settings: &State) -> Result<(), Error> {
    let saved = SavedSettings {
        save_dir: settings.save_dir.clone(),
        game_dir: settings.game_dir.clone(),
    };

    let f = std::fs::File::create(get_cache_file_path())?;
    let writer = std::io::BufWriter::new(f);

    serde_json::to_writer(writer, &saved).map_err(Error::Serialization)
}

fn read_settings() -> Result<SavedSettings, Error> {
    let f = std::fs::File::open(get_cache_file_path())?;
    let reader = std::io::BufReader::new(f);

    serde_json::from_reader(reader).map_err(Error::Deserialization)
}

fn path_to_string(path: &Option<PathBuf>) -> String {
    path.as_deref()
        .and_then(|x| x.to_str())
        .map(|x| x.to_string())
        .unwrap_or_default()
}

#[derive(Debug)]
pub struct State {
    pub active: bool,
    pub game_dir: Option<PathBuf>,
    pub save_dir: Option<PathBuf>,

    game_dir_temp: String,
    save_dir_temp: String,
}
impl Default for State {
    fn default() -> Self {
        match read_settings() {
            Ok(settings) => Self {
                active: false,
                game_dir_temp: path_to_string(&settings.game_dir),
                save_dir_temp: path_to_string(&settings.save_dir),
                game_dir: settings.game_dir,
                save_dir: settings.save_dir,
            },
            Err(_) => Self {
                active: false,
                game_dir: None,
                save_dir: None,

                game_dir_temp: String::new(),
                save_dir_temp: String::new(),
            },
        }
    }
}
impl State {
    pub fn is_unset(&self) -> bool {
        self.game_dir.is_none() || self.save_dir.is_none()
    }

    fn close(&mut self) {
        self.active = false;
        self.game_dir_temp = path_to_string(&self.game_dir);
        self.save_dir_temp = path_to_string(&self.save_dir);
    }

    fn pick_dir(&mut self, mode: PickDirMode) {
        let initial_dir = {
            let path = match mode {
                PickDirMode::Save => &self.save_dir_temp,
                PickDirMode::Game => &self.game_dir_temp,
            };

            if path.is_empty() {
                None
            } else {
                Some(Path::new(path))
            }
        };

        let mut dialog = rfd::FileDialog::new();

        if let Some(dir) = initial_dir {
            dialog = dialog.set_directory(dir);
        }

        let folder = dialog.pick_folder();
        if let Some(folder) = folder {
            match mode {
                PickDirMode::Game => self.game_dir_temp = folder.to_string_lossy().into_owned(),
                PickDirMode::Save => self.save_dir_temp = folder.to_string_lossy().into_owned(),
            }
        }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::SetGameDir(dir) => self.game_dir_temp = dir,
            Message::SetSaveDir(dir) => self.save_dir_temp = dir,
            Message::Close => {
                self.close();
            }
            Message::Save => {
                let set_dir = |d: &mut Option<PathBuf>, p: &str| {
                    let path = Some(PathBuf::from(p));
                    *d = path;
                };

                set_dir(&mut self.game_dir, &self.game_dir_temp);
                set_dir(&mut self.save_dir, &self.save_dir_temp);

                save_settings(self).expect("Failed to save settings");

                self.close();
            }
            Message::PickDir(mode) => self.pick_dir(mode),
        }
    }

    pub fn view(&self) -> Element<'_> {
        let game_dir = self.game_dir_temp.as_str();
        let save_dir = self.save_dir_temp.as_str();

        let body = column![
            text("Game Directory"),
            row![
                text_input("Game Directory", game_dir).on_input(Message::SetGameDir),
                button("...").on_press(Message::PickDir(PickDirMode::Game)),
            ]
            .spacing(8),
            vertical_space().height(16),
            text("Save Directory"),
            row![
                text_input("Save Directory", save_dir).on_input(Message::SetSaveDir),
                button("...").on_press(Message::PickDir(PickDirMode::Save)),
            ]
            .spacing(8),
            vertical_space().height(Length::Fill),
            row![
                horizontal_space().width(Length::Fill),
                button("Close").on_press(Message::Close),
                button("Save").on_press(Message::Save),
            ]
            .spacing(16),
        ]
        .spacing(8);

        super::bordered(body.into())
    }
}
