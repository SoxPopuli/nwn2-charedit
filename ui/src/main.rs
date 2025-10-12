mod error;
mod ids;

use crate::error::Error;
use iced::{
    Task,
    widget::{button, column, row, text},
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
    pub fn new(
        field: StructField,
        expect_fn: impl FnOnce(&Field) -> Result<T, nwn_lib::error::Error>,
    ) -> Result<Self, Error> {
        let lock = field.read()?;
        let value = expect_fn(&lock.field)?;
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
pub struct Player {
    pub first_name: FieldRef<String>,
    pub last_name: FieldRef<String>,
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

impl Player {
    pub fn new(player_struct: &Struct) -> Result<Self, Error> {
        let read_name = |field: &Field| {
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

            match label.as_str() {
                "FirstName" => {
                    player_builder.first_name(FieldRef::new(field.clone(), read_name)?);
                }
                "LastName" => {
                    player_builder.last_name(FieldRef::new(field.clone(), read_name)?);
                }
                "Str" => {
                    player_builder.str(FieldRef::new(field.clone(), Field::expect_byte)?);
                }
                "Dex" => {
                    player_builder.dex(FieldRef::new(field.clone(), Field::expect_byte)?);
                }
                "Con" => {
                    player_builder.con(FieldRef::new(field.clone(), Field::expect_byte)?);
                }
                "Int" => {
                    player_builder.int(FieldRef::new(field.clone(), Field::expect_byte)?);
                }
                "Wis" => {
                    player_builder.wis(FieldRef::new(field.clone(), Field::expect_byte)?);
                }
                "Cha" => {
                    player_builder.cha(FieldRef::new(field.clone(), Field::expect_byte)?);
                }
                "GoodEvil" => {
                    player_builder.good_evil(FieldRef::new(field.clone(), Field::expect_byte)?);
                }
                "LawfulChaotic" => {
                    player_builder
                        .lawful_chaotic(FieldRef::new(field.clone(), Field::expect_byte)?);
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
    players: Vec<Player>,
}
impl SaveFile {
    pub fn new(file: Gff) -> Self {
        let player_list = file
            .root
            .bfs_iter()
            .find(|x| x.has_label("Mod_PlayerList"))
            .expect("Couldn't find player list");

        let player_list = {
            let lock = player_list.read().unwrap();
            lock.field.expect_list().cloned().unwrap()
        };

        let players: Vec<Player> = player_list
            .iter()
            .map(Player::new)
            .map_while(Result::ok)
            .collect();

        Self { file, players }
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
            Message::NoMsg => Task::none(),
            Message::OpenFileDialog => {
                let file = rfd::FileDialog::new()
                    .set_title("Open save file")
                    .add_filter("Save File (gffres.zip, playerlist.ifo)", &["zip", "ifo"])
                    .pick_file();

                match file {
                    Some(path) => Task::done(Message::FileSelected(path)),
                    None => Task::none(),
                }
            }

            Message::FileSelected(path) => {
                let save =
                    open_file(&path).unwrap_or_else(|e| panic!("Failed to open save file: {e}"));

                self.save_file = Some(SaveFile::new(save));

                Task::none()
            }
        }
    }

    fn menu(&self) -> Element<'_> {
        use iced_aw::menu::{Item, Menu};
        use iced_aw::{menu_bar, menu_items};

        let menu_template = |items| Menu::new(items).max_width(80.0).offset(6.0);

        macro_rules! menu {
            ($($x:tt)+) => {
                menu_template(menu_items!( $($x)+ ))
            };
        }

        let menu_bar = menu_bar!((
            menu_button("File").on_press(Message::NoMsg),
            menu!((menu_button("Open").on_press(Message::OpenFileDialog))(
                menu_button("Save")
            ))
        ));

        column![menu_bar, iced::widget::horizontal_rule(4),]
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

        let body = iced::widget::Column::with_children(names).padding(iced::Padding {
            top: 0.0,
            ..(16.0).into()
        });

        let icon = {
            // let file = include_bytes!("../../lib/src/tests/files/is_fireball.dds");

            // let dds = ddsfile::Dds::read(file.as_slice()).unwrap();

            // let data = dds.get_data(0).unwrap();
            // let width = dds.get_width();
            // let height = dds.get_height();
            // dbg!((width, height));
            // let handle = iced::widget::image::Handle::from_rgba(width, height, data.to_vec());

            iced::widget::container(
                iced::widget::Image::new(handle),
            )
            .style(iced::widget::container::bordered_box)
        };

        column![self.menu(), body, icon].into()
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
mod tests {
    #[test]
    fn ros_test() {
        let tlk_file = {
            let file = include_bytes!(
                "/home/charlotte/.local/share/Steam/steamapps/common/NWN2 Enhanced Edition/dialog.tlk"
            );
            let file = std::io::Cursor::new(file);

            nwn_lib::files::tlk::Tlk::read(file).unwrap()
        };

        // let file = include_bytes!("../../files/npc_bevil.ros");
        let file = include_bytes!("../../files/roster.rst");
        let file = std::io::Cursor::new(file);
        let gff = super::Gff::read(file, Some(&tlk_file)).unwrap();

        panic!("{gff:#?}");
    }

    #[test]
    fn spells() {
        let tlk_file = {
            let file = include_bytes!(
                "/home/charlotte/.local/share/Steam/steamapps/common/NWN2 Enhanced Edition/dialog.tlk"
            );
            let file = std::io::Cursor::new(file);

            nwn_lib::files::tlk::Tlk::read(file).unwrap()
        };

        let spells = {
            let file = include_bytes!(
                "/home/charlotte/.local/share/Steam/steamapps/common/NWN2 Enhanced Edition/data/2DA/spells.2da"
            );

            nwn_lib::files::two_da::parse(file.as_slice()).unwrap()
        };

        let icon_index = spells.find_column_index("IconResRef").unwrap();
        let name_index = 0;
        let description_index = spells.find_column_index("SpellDesc").unwrap();

        let icons = spells.get_column_data(icon_index);
        let names = spells.get_column_data(name_index);
        let descriptions = spells.get_column_data(description_index).map(|x| {
            x.and_then(|x| {
                let r = x.parse().ok()?;
                tlk_file.get_from_str_ref(r).ok()
            })
        });

        let x = icons
            .zip(names)
            .zip(descriptions)
            .map(|x| {
                let ((a, b), c) = x;
                (a, b, c)
            })
            .filter_map(|x| match x {
                (Some(a), Some(b), Some(c)) => Some((a, b, c)),
                _ => None,
            })
            .collect::<Vec<_>>();

        panic!("{x:#?}");
    }
}
