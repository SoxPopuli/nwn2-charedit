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
    error::Error,
    player::{Player, PlayerClass},
    two_d_array::FileReader2DA,
    ui::settings::GameResources,
};
use iced::{
    Task,
    widget::{button, column, row, text, vertical_space},
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
    NoMsg,
    FileSelected(PathBuf),
    Settings(ui::SettingsMessage),
    Character(ui::CharacterMessage),
    OpenSettings,
    OpenFileSelector,
    FileSelector(ui::SelectFileMessage),
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
pub struct SaveFile(pub Gff);
impl SaveFile {
    pub fn get_players(&self, tlk: &Tlk, reader_2da: &mut FileReader2DA) -> Vec<Player> {
        let player_list = self
            .0
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
        Ok(self.0.write(output)?)
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

fn view_class_spells<'a>(
    class: &'a PlayerClass,
    spell_record: &'a spell::SpellRecord,
) -> Option<Element<'a>> {
    use iced_aw::{Tabs, tab_bar::TabLabel};

    if !class.is_caster {
        return None;
    }

    let mut tabs = Tabs::new(|_| Message::NoMsg);

    let spells = class.spell_known_list.iter().flatten().enumerate();

    for (level, spells_known) in spells {
        let spells = spells_known.spells.iter().map(|spell| {
            let spell = spell_record
                .spells
                .get(&(spell.0 as usize))
                .unwrap_or_else(|| panic!("{}: {} not found", spell, spell.0));

            let image: Element<'_> = match spell.icon.as_ref() {
                Some(i) => iced::widget::image(i).width(40.0).into(),
                None => vertical_space().width(40.0).into(),
            };

            let desc: Element<'_> = match spell.desc.as_ref() {
                Some(x) => text(&x.data).into(),
                None => vertical_space().into(),
            };

            row![image, text(&spell.name.data).width(80.0), desc,]
                .spacing(16)
                .into()
        });

        let spells = iced::widget::Column::from_iter(spells).spacing(16.0);
        let spells = iced::widget::scrollable(spells);

        tabs = tabs.push(level, TabLabel::Text(level.to_string()), spells);
    }

    Some(tabs.into())
}

#[derive(Debug)]
struct App {
    pub save_file: Option<SaveFile>,
    pub characters: ui::CharacterState,
    pub settings: ui::SettingsState,
    pub select_file: ui::SelectFileState,
}
impl App {
    fn title() -> &'static str {
        env!("CARGO_BIN_NAME")
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
        };

        (this, Task::none())
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::NoMsg => {}
            Message::FileSelected(path) => match open_file(&path) {
                Ok(save) => {
                    match self.settings.game_resources.as_mut() {
                        Some(g) => {
                            let save_file = SaveFile(save);

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
            Message::Character(msg) => {
                self.characters.update(msg);
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

    fn view_player<'a>(&'a self, p: &'a Player) -> Element<'a> {
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

        let spells_panel = p.classes.iter().find_map(|x| {
            view_class_spells(
                x,
                &self.settings.game_resources.as_ref().unwrap().spell_record,
            )
        });

        row![stats].push_maybe(spells_panel).into()
    }

    fn view(&self) -> Element<'_> {
        let body = if self.settings.active {
            self.settings.view().map(Message::Settings)
        } else if self.select_file.active {
            self.select_file.view().map(Message::FileSelector)
        } else {
            let (spell_record, feat_record) = match &self.settings.game_resources {
                Some(GameResources {
                    spell_record,
                    feat_record,
                    ..
                }) => (spell_record, feat_record),
                None => return text("Game Directory not set correctly").into(),
            };

            self.characters
                .view(spell_record, feat_record)
                .map(Message::Character)
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
