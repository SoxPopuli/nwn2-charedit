use std::path::{Path, PathBuf};

use iced::{
    Length,
    widget::{button, column, horizontal_space, row, text, text_input, vertical_space},
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

#[derive(Debug, Default)]
pub struct State {
    pub active: bool,
    pub game_dir: Option<PathBuf>,
    pub save_dir: Option<PathBuf>,

    game_dir_temp: String,
    save_dir_temp: String,
}
impl State {
    pub fn is_unset(&self) -> bool {
        self.game_dir.is_none() || self.save_dir.is_none()
    }

    fn close(&mut self) {
        let path_to_string = |path: &Option<PathBuf>| {
            path.as_deref()
                .and_then(|x| x.to_str())
                .map(|x| x.to_string())
                .unwrap_or_default()
        };

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
