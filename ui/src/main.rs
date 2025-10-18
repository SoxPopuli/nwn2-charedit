mod error;
mod ids;
mod two_d_array;
mod ui;

use crate::{error::Error, ui::settings::Message as SettingsMessage};
use iced::{
    Task,
    widget::{Column, button, column, row, text},
};
use nwn_lib::files::gff::{
    Gff,
    field::Field,
    r#struct::{Struct, StructField},
};
use std::{
    fs::File,
    io::Read,
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
    OpenFileDialog,
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

#[derive(Debug, Clone)]
pub struct FieldRef<T> {
    field: StructField,
    value: T,
}
impl<T> FieldRef<T> {
    pub fn new<E>(
        field: StructField,
        expect_fn: impl FnOnce(&Field) -> Result<T, E>,
    ) -> Result<Self, Error>
    where
        E: Into<Error>,
    {
        let lock = field.read()?;
        let value = expect_fn(&lock.field).map_err(|e| e.into())?;
        drop(lock);

        Ok(Self {
            field: field.clone(),
            value,
        })
    }

    pub fn set(&mut self, new_value: T, save_fn: impl FnOnce(&T) -> Field) {
        self.value = new_value;

        let mut lock = self.field.write().unwrap();
        lock.field = save_fn(&self.value);
    }

    pub fn get(&self) -> &T {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub struct Alignment {
    pub good_evil: FieldRef<u8>,
    pub lawful_chaotic: FieldRef<u8>,
}
impl std::fmt::Display for Alignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let good_evil = self.good_evil.get();
        let lawful_chaotic = self.lawful_chaotic.get();

        let good_evil = match good_evil {
            70..=100 => Some("Good"),
            31..=69 => Some("Neutral"),
            0..=30 => Some("Evil"),
            _ => None,
        };

        let lawful_chaotic = match lawful_chaotic {
            70..=100 => Some("Lawful"),
            31..=69 => Some("Neutral"),
            0..=30 => Some("Chaotic"),
            _ => None,
        };

        match (good_evil, lawful_chaotic) {
            (Some(ge), Some(lc)) => write!(f, "{lc} {ge}"),
            (Some(ge), None) => write!(f, "{ge}"),
            (None, Some(lc)) => write!(f, "{lc}"),
            (None, None) => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attributes {
    pub str: FieldRef<u8>,
    pub dex: FieldRef<u8>,
    pub con: FieldRef<u8>,
    pub int: FieldRef<u8>,
    pub wis: FieldRef<u8>,
    pub cha: FieldRef<u8>,
}

#[derive(Debug, Clone)]
pub struct Race {
    pub race: String,
    pub subrace: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub first_name: FieldRef<String>,
    pub last_name: FieldRef<String>,
    pub race: Race,
    pub attributes: Attributes,
    pub alignment: Alignment,
}

macro_rules! make_builder {
    (struct $name: ident { $($field: ident : $t: ty),+ $(,)* }) => {
        #[derive(Debug, Default)]
        pub struct $name {
            $(pub $field : Option<$t>),+
        }
        impl $name {
            $(
                pub fn $field (&mut self, x: $t) {
                    self.$field = Some(x);
                }
            )+
        }
    };
}

make_builder! {
    struct PlayerBuilder {
        first_name: FieldRef<String>,
        last_name: FieldRef<String>,
        race: FieldRef<String>,
        subrace: FieldRef<String>,
        str: FieldRef<u8>,
        dex: FieldRef<u8>,
        con: FieldRef<u8>,
        int: FieldRef<u8>,
        wis: FieldRef<u8>,
        cha: FieldRef<u8>,
        good_evil: FieldRef<u8>,
        lawful_chaotic: FieldRef<u8>,
    }
}

impl PlayerBuilder {
    fn build(self) -> Result<Player, Error> {
        macro_rules! unwrap_field {
            ($field: ident) => {
                self.$field
                    .ok_or($crate::error::Error::MissingField(format!(
                        "Missing field {} in player builder",
                        stringify!($field)
                    )))?
            };
        }

        Ok(Player {
            first_name: unwrap_field!(first_name),
            last_name: unwrap_field!(last_name),
            race: Race {
                race: unwrap_field!(race).value,
                subrace: self.subrace.map(|x| x.value),
            },
            attributes: Attributes {
                str: unwrap_field!(str),
                dex: unwrap_field!(dex),
                con: unwrap_field!(con),
                int: unwrap_field!(int),
                wis: unwrap_field!(wis),
                cha: unwrap_field!(cha),
            },
            alignment: Alignment {
                good_evil: unwrap_field!(good_evil),
                lawful_chaotic: unwrap_field!(lawful_chaotic),
            },
        })
    }
}

type Tlk = nwn_lib::files::tlk::Tlk<File>;

fn get_race_name_from_id(
    tlk: &Tlk,
    reader: &mut two_d_array::FileReader,
    field: &Field,
) -> Result<String, Error> {
    let file_name = "racialtypes.2da";
    let table = reader.read(file_name)?;
    let race_id = field.expect_byte()?;
    let name_idx = table
        .find_column_index("Name")
        .ok_or(Error::MissingField(format!(
            "Missing 'Name' field in {file_name}"
        )))?;

    let s_ref = table.data[(name_idx, race_id as usize)]
        .clone()
        .ok_or(Error::MissingField(format!(
            "Missing race name in {file_name} for {race_id}"
        )))?;

    let x = tlk
        .get_from_str_ref(
            s_ref
                .parse()
                .map_err(|e: std::num::ParseIntError| Error::MissingField(e.to_string()))?,
        )
        .map_err(Error::LibError)
        .and_then(|x| x.ok_or(Error::MissingField("Missing race name str_ref".into())))?;

    Ok(x.to_string())
}

impl Player {
    pub fn new(
        tlk: &Tlk,
        data_reader: &mut two_d_array::FileReader,
        player_struct: &Struct,
    ) -> Result<Self, Error> {
        let read_name = |field: &Field| -> Result<String, Error> {
            let s = field.expect_exolocstring()?;
            Ok(s.substrings
                .iter()
                .map(|sub| &sub.data)
                .fold(String::new(), |acc, x| acc + x))
        };

        let mut player_builder = PlayerBuilder::default();

        for field in &player_struct.fields {
            let lock = field.read()?;
            let label = &lock.label;

            macro_rules! read_field {
                ($builder_fn:ident, $expect_fn:expr) => {{ player_builder.$builder_fn(FieldRef::new(field.clone(), $expect_fn)?) }};
            }

            match label.as_str() {
                "FirstName" => read_field!(first_name, read_name),
                "LastName" => read_field!(last_name, read_name),
                "Race" => read_field!(race, |f| get_race_name_from_id(tlk, data_reader, f)),
                "Gender" => {}
                "Subrace" => {}
                "Str" => read_field!(str, Field::expect_byte),
                "Dex" => read_field!(dex, Field::expect_byte),
                "Con" => read_field!(con, Field::expect_byte),
                "Int" => read_field!(int, Field::expect_byte),
                "Wis" => read_field!(wis, Field::expect_byte),
                "Cha" => read_field!(cha, Field::expect_byte),
                "GoodEvil" => read_field!(good_evil, Field::expect_byte),
                "LawfulChaotic" => read_field!(lawful_chaotic, Field::expect_byte),
                "LvlStatList" => {
                    // let lock = field.read().unwrap();
                    // let s = lock.field.expect_list().unwrap();
                }

                _ => {}
            }
        }

        player_builder.build()
    }
}

#[derive(Debug)]
pub struct SaveFile {
    file: Gff,
    tlk: Tlk,
    players: Vec<Player>,
    data_reader: two_d_array::FileReader,
}
impl SaveFile {
    pub fn new(file: Gff, tlk: Tlk) -> Self {
        let player_list = file
            .root
            .bfs_iter()
            .find(|x| x.has_label("Mod_PlayerList"))
            .expect("Couldn't find player list");

        let player_list = {
            let lock = player_list.read().unwrap();
            lock.field.expect_list().cloned().unwrap()
        };

        let mut reader = two_d_array::FileReader::new().expect("Failed to create 2da reader");

        let players: Vec<Player> = player_list
            .iter()
            .map(|x| Player::new(&tlk, &mut reader, x))
            .map_while(Result::ok)
            .collect();

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
            Message::OpenFileDialog => {
                let file = rfd::FileDialog::new()
                    .set_title("Open save file")
                    .add_filter("Save File (gffres.zip, playerlist.ifo)", &["zip", "ifo"])
                    .pick_file();

                if let Some(path) = file {
                    return Task::done(Message::FileSelected(path));
                }
            }

            Message::FileSelected(path) => {
                let save =
                    open_file(&path).unwrap_or_else(|e| panic!("Failed to open save file: {e}"));

                // self.save_file = Some(SaveFile::new(save));
            }
            Message::Settings(m) => {
                self.settings.update(m);
            }
            Message::OpenSettings => {
                self.settings.active = true;
            }
            Message::OpenFileSelector => {
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
                self.select_file.update(m);
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

            column![
                text(format!("{} {}", p.first_name.get(), p.last_name.get())),
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
            ]
            .into()
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

fn read_dir_recursive(path: &std::path::Path) -> impl Iterator<Item = PathBuf> {
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
