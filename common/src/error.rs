#[derive(Debug)]
pub struct EnumError {
    pub enum_type: &'static str,
    pub msg: String,
}
impl std::fmt::Display for EnumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for EnumError {}
