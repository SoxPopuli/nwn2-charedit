use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    ParseError(String),
    WriteError(String),
    InvalidStrRef {
        value: u32,
    },
    EnumError {
        enum_type: &'static str,
        msg: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}

impl From<roxmltree::Error> for Error {
    fn from(value: roxmltree::Error) -> Self {
        Self::ParseError(value.to_string())
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::ParseError(value.to_string())
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Self::ParseError(value.to_string())
    }
}

#[derive(Debug)]
pub struct FileError {
    pub file: String,
    pub err: Error,
}
impl FileError {
    pub fn from_err(file_name: String, e: Error) -> Self {
        Self {
            file: file_name,
            err: e,
        }
    }

    pub fn from_result<T>(file_name: String, res: Result<T, Error>) -> Result<T, Self> {
        res.map_err(|e| Self {
            file: file_name,
            err: e,
        })
    }
}

pub trait IntoError<T> {
    fn into_parse_error(self) -> Result<T, Error>;
    fn into_write_error(self) -> Result<T, Error>;
}

impl<T, E> IntoError<T> for Result<T, E>
where
    E: std::error::Error,
{
    fn into_parse_error(self) -> Result<T, Error> {
        self.map_err(|e| Error::ParseError(e.to_string()))
    }

    fn into_write_error(self) -> Result<T, Error> {
        self.map_err(|e| Error::WriteError(e.to_string()))
    }
}
