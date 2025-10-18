use std::{path::PathBuf, sync::PoisonError};

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    Serialization(serde_json::Error),
    Deserialization(serde_json::Error),
    EnvNotFound {
        var: &'static str,
    },
    MissingGamePath(PathBuf),
    MissingDialogFile,
    Io(std::io::ErrorKind),
    LibError(nwn_lib::error::Error),
    LockError(String),
    FieldExpectError {
        field_name: &'static str,
        error: nwn_lib::error::Error,
    },
    MissingField(String),
    ParseError(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDialogFile => write!(f, "Couldn't find dialog.tlk in game directory"),
            x => write!(f, "{:?}", x),
        }
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
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.kind())
    }
}
