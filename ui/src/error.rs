use std::sync::PoisonError;

#[derive(Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    LibError(nwn_lib::error::Error),
    LockError(String),
    FieldExpectError {
        field_name: &'static str,
        error: nwn_lib::error::Error,
    },
    MissingField(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
impl From<nwn_lib::error::Error> for Error {
    fn from(value: nwn_lib::error::Error) -> Self {
        Self::LibError(value)
    }
}
impl<T> From<PoisonError<T>> for Error {
    fn from(value: PoisonError<T>) -> Self {
        Self::LockError(value.to_string())
    }
}
