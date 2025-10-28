use crate::SaveFile;

#[derive(Debug, Clone)]
pub enum Message {
}

type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Default, Clone)]
pub struct State {
    pub active: bool,
}
impl State {
    pub fn update(&mut self, msg: Message) {

    }

    pub fn view(&self, save_file: &SaveFile) -> Element<'_> {
        let save_dir = save_file.path.parent().expect("Failed to get save file dir");

        todo!()
    }
}
