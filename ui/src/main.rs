mod error;
mod field_ref;
mod ids;
mod player;
mod two_d_array;
mod ui;

use crate::{
    error::Error, player::Player, player::PlayerClass, ui::settings::Message as SettingsMessage,
};
use iced::{
    Task,
    widget::{Column, button, column, row, text},
};
use nwn_lib::files::gff::Gff;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

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
    NoMsg,
    FileSelected(PathBuf),
    Settings(SettingsMessage),
    OpenSettings,
    OpenFileSelector,
    FileSelector(ui::select_file::Message),
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
            text_color: palette.text,
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
    pub tlk: Tlk,
    pub players: Vec<Player>,
    pub data_reader: two_d_array::FileReader,
}
impl SaveFile {
    pub fn new(file: Gff, tlk: Tlk) -> Self {
        let player_list = file
            .root
            .bfs_iter()
            .find(|x| x.has_label("Mod_PlayerList"))
            .expect("Couldn't find player list");

        let lock = player_list.read().unwrap();
        let player_list = lock.field.expect_list().unwrap();

        let mut reader = two_d_array::FileReader::new().expect("Failed to create 2da reader");

        let players = player_list
            .iter()
            .map(|x| Player::new(&tlk, &mut reader, x))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        Self {
            file,
            tlk,
            players,
            data_reader: reader,
        }
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
        None => Err(Error::MissingDialogFile),
    }
}

fn view_class_spells(class: &PlayerClass) -> Option<Element<'_>> {
    use iced_aw::{Tabs, tab_bar::TabLabel};

    if !class.is_caster {
        return None;
    }

    let mut tabs = Tabs::new(|_| Message::NoMsg);

    let spells = class.spell_known_list.iter().flatten().enumerate();

    for (i, _spells) in spells {
        tabs = tabs.push(i, TabLabel::Text(i.to_string()), text(i));
    }

    Some(tabs.into())
}

#[derive(Debug, Default)]
struct App {
    save_file: Option<SaveFile>,
    settings: ui::settings::State,
    select_file: ui::select_file::State,
}
impl App {
    fn title() -> &'static str {
        env!("CARGO_BIN_NAME")
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::NoMsg => {}
            Message::FileSelected(path) => match open_file(&path) {
                Ok(save) => {
                    let tlk = match self.settings.game_dir.as_deref().map(get_tlk_file) {
                        Some(Ok(file)) => file,
                        Some(Err(e)) => return show_error_popup_task(e.to_string()),
                        None => return show_error_popup_task("Game Directory not set"),
                    };

                    self.save_file = Some(SaveFile::new(save, tlk))
                }
                Err(e) => show_error_popup(format!("Failed to open save file: {e}")),
            },
            Message::Settings(m) => {
                self.settings.update(m);
            }
            Message::OpenSettings => {
                self.settings.active = true;
                self.select_file.active = false;
            }
            Message::OpenFileSelector => {
                if let Some(dir) = &self.settings.save_dir {
                    self.select_file.open(dir);
                    self.settings.close();
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
        }

        Task::none()
    }

    fn menu(&self) -> Element<'_> {
        let settings = menu_button("Settings").on_press(Message::OpenSettings);

        let open_file = menu_button("Open").on_press(Message::OpenFileSelector);
        let menu_bar = row![open_file, settings].spacing(8);

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
        fn view_player(p: &Player) -> Element<'_> {
            fn row(name: &str, value: impl std::fmt::Display) -> iced_aw::GridRow<'_, Message> {
                iced_aw::grid_row![text(format!("{name}:")), text(value.to_string())]
            }

            let classes = {
                let classes = p
                    .classes
                    .iter()
                    .map(|class| format!("{} ({})", class.class.value, class.level.value))
                    .collect::<Vec<_>>();

                let c = classes.join(" | ");

                text(c)
            };

            let stats = column![
                text(format!("{} {}", p.first_name.get(), p.last_name.get())),
                text(p.gender.to_string()),
                text(p.race.to_string()),
                classes,
                iced_aw::grid![
                    row("Strength", p.attributes.str.get()),
                    row("Dexterity", p.attributes.dex.get()),
                    row("Constitution", p.attributes.con.get()),
                    row("Intelligence", p.attributes.int.get()),
                    row("Wisdom", p.attributes.wis.get()),
                    row("Charisma", p.attributes.cha.get()),
                    row("Alignment", &p.alignment)
                ]
                .column_spacing(20),
            ];

            let spells_panel = p.classes.iter().find_map(view_class_spells);

            row![stats].push_maybe(spells_panel).into()
        }

        let names = match &self.save_file {
            Some(save) => save.players.iter().map(view_player).collect(),
            None => Vec::new(),
        };

        let body = if self.settings.active {
            self.settings.view().map(Message::Settings)
        } else if self.select_file.active {
            self.select_file.view().map(Message::FileSelector)
        } else {
            Column::with_children(names)
                .padding(iced::Padding {
                    top: 0.0,
                    ..(16.0).into()
                })
                .into()
        };

        column![self.menu(), body].into()
    }

    fn run() -> Result<(), iced::Error> {
        iced::application(Self::title(), Self::update, Self::view)
            .centered()
            .window_size((640.0, 480.0))
            .theme(Self::theme)
            .run()
    }
}

fn main() {
    App::run().unwrap()
}

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

#[cfg(test)]
mod tests {}
