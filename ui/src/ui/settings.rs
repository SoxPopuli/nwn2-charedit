use crate::{
    Tlk, error::Error, feat::FeatRecord, popup_opt, popup_panic, show_error_popup,
    spell::SpellRecord, two_d_array::FileReader2DA,
};
use cfg_if::cfg_if;
use iced::{
    Length,
    widget::{button, column, horizontal_space, row, text, text_input, vertical_space},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

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
        game_dir: settings
            .game_resources
            .as_ref()
            .map(|GameResources { game_dir, .. }| game_dir.clone()),
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

fn path_to_string(path: Option<&Path>) -> String {
    path.and_then(|x| x.to_str())
        .map(|x| x.to_string())
        .unwrap_or_default()
}

pub type IconName = String;
pub type IconPath = PathBuf;

pub(crate) fn read_dir_recursive(path: &std::path::Path) -> impl Iterator<Item = PathBuf> {
    use std::collections::VecDeque;
    use std::fs::DirEntry;
    use std::path::Path;

    fn read_dir(dir: impl AsRef<Path>) -> impl Iterator<Item = DirEntry> {
        dir.as_ref().read_dir().unwrap().filter_map(Result::ok)
    }

    let mut stack = VecDeque::from_iter(read_dir(path));

    std::iter::from_fn(move || {
        while let Some(x) = stack.pop_front() {
            let metadata = x.metadata().unwrap();
            if metadata.is_file() {
                return Some(x.path());
            } else if metadata.is_dir() {
                read_dir(x.path()).for_each(|x| stack.push_front(x));
            }
        }

        None
    })
}

fn get_icon_paths(game_dir: &Path) -> HashMap<IconName, IconPath> {
    read_dir_recursive(game_dir)
        .filter_map(|x| {
            let name = x
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(|x| x.to_string())?;
            Some((name, x))
        })
        .collect()
}

fn get_tlk_file(game_dir: &Path) -> Result<Tlk, Error> {
    let mut read_dir = game_dir.read_dir()?;

    let file_path = read_dir.find_map(|x| {
        if let Ok(dir) = x
            && let Ok(m) = dir.metadata()
            && m.is_file()
            && dir.file_name().eq_ignore_ascii_case("dialog.tlk")
        {
            return Some(dir.path());
        }

        None
    });

    match file_path {
        Some(p) => {
            let f = File::open(p)?;
            Tlk::read(BufReader::new(f)).map_err(Error::LibError)
        }
        None => Err(Error::MissingDialogFile(game_dir.into())),
    }
}

#[derive(Debug)]
pub struct GameResources {
    pub game_dir: PathBuf,
    pub tlk: Tlk,
    pub icon_paths: HashMap<IconName, IconPath>,
    pub feat_record: FeatRecord,
    pub spell_record: SpellRecord,
    pub file_reader: FileReader2DA,
}
impl GameResources {
    fn load(game_dir: &Path) -> Result<Self, Error> {
        let tlk = get_tlk_file(game_dir)?;
        let icon_paths = get_icon_paths(game_dir);

        let mut reader = FileReader2DA::new(game_dir)?;

        let feat_record = FeatRecord::new(&tlk, &mut reader, &icon_paths)?;
        let spell_record = SpellRecord::new(&tlk, game_dir, &icon_paths)?;

        Ok(Self {
            game_dir: game_dir.into(),
            tlk,
            icon_paths,
            feat_record,
            spell_record,
            file_reader: reader,
        })
    }
}

#[derive(Debug)]
pub struct State {
    pub active: bool,
    pub save_dir: Option<PathBuf>,
    pub game_resources: Option<GameResources>,

    game_dir_temp: String,
    save_dir_temp: String,
}
impl State {
    pub fn from_file_or_default() -> Self {
        match read_settings() {
            Ok(settings) => Self {
                active: false,
                game_dir_temp: path_to_string(settings.game_dir.as_deref()),
                save_dir_temp: path_to_string(settings.save_dir.as_deref()),

                game_resources: match settings.game_dir.as_deref().map(GameResources::load) {
                    Some(Ok(x)) => Some(x),
                    Some(Err(e)) => {
                        show_error_popup(e.to_string());
                        None
                    }
                    None => None,
                },
                save_dir: settings.save_dir,
            },
            Err(_) => Self {
                active: false,
                save_dir: None,
                game_resources: None,

                game_dir_temp: String::new(),
                save_dir_temp: String::new(),
            },
        }
    }

    pub fn close(&mut self) {
        self.active = false;

        let game_dir = self
            .game_resources
            .as_ref()
            .map(|GameResources { game_dir, .. }| game_dir.as_path());

        self.game_dir_temp = path_to_string(game_dir);
        self.save_dir_temp = path_to_string(self.save_dir.as_deref());
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
                let game_dir = Path::new(&self.game_dir_temp);
                self.game_resources = match GameResources::load(game_dir) {
                    Ok(x) => Some(x),
                    Err(e) => popup_opt!("{e}"),
                };

                self.save_dir = Some(PathBuf::from(&self.save_dir_temp));

                save_settings(self)
                    .unwrap_or_else(|e| popup_panic!("Failed to save settings: {e}"));

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
            .height(Length::Fixed(32.0))
            .spacing(16),
        ]
        .spacing(8);

        super::bordered(body.into())
    }
}
