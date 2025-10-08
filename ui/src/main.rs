mod error;
mod ids;

use crate::error::Error;
use iced::{
    Task,
    widget::{button, column, text},
};
use nwn_lib::files::gff::{Gff, field::Field, r#struct::Struct};
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

#[derive(Debug)]
pub struct Player {
    pub first_name: String,
    pub last_name: String,
}
impl Player {
    pub fn new(player_struct: &Struct) -> Self {
        let read_name = |field: &Field| {
            let s = field.expect_exolocstring().ok()?;
            Some(
                s.substrings
                    .iter()
                    .map(|sub| &sub.data)
                    .fold(String::new(), |acc, x| acc + x),
            )
        };

        let first_name = player_struct
            .bfs_iter()
            .find(|x| x.has_label("FirstName"))
            .and_then(|field| field.read_field(read_name))
            .expect("Couldn't find first name");

        let last_name = player_struct
            .bfs_iter()
            .find(|x| x.has_label("LastName"))
            .and_then(|field| field.read_field(read_name))
            .expect("Couldn't find last name");

        Self {
            first_name,
            last_name,
        }
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

        let players = player_list.iter().map(Player::new).collect();

        // println!("{player_list:#?}");

        Self { file, players }
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
        use iced_aw::menu::{Item, Menu, MenuBar};
        use iced_aw::{menu_bar, menu_items};

        let menu_template = |items| Menu::new(items).max_width(180.0).offset(6.0);

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
        let names = match &self.save_file {
            Some(save) => save
                .players
                .iter()
                .map(|p| format!("{} {}", p.first_name, p.last_name))
                .map(text)
                .map(|x| x.into())
                .collect(),
            None => Vec::new(),
        };

        let body = iced::widget::Column::with_children(names).padding(8.0);

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
