pub enum Message {
    
}

type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Default, Clone)]
pub struct State {

}
impl State {
    pub fn update(&mut self, msg: Message) {
        
    }

    pub fn view(&self) -> Element<'_> {
        todo!()
    }
}
