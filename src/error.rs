use std::{
    num::{ParseFloatError, ParseIntError},
    str::FromStr,
};

trait ReadFile<'a>: std::io::Read {
    fn file_name() -> &'a str;
}

#[derive(Debug)]
pub enum Error {
    ParseError(String),
}

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
