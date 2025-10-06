#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    LibError(nwn_lib::error::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
impl std::error::Error for Error {}
impl From<nwn_lib::error::Error> for Error {
    fn from(value: nwn_lib::error::Error) -> Self {
        Self::LibError(value)
    }
}
