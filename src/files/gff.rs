use crate::error::Error::*;
use std::io::Read;

macro_rules! int_enum {
    ($name: ident, $($case: ident, $val: expr),+) => {
        pub enum $name {
            $($case = $val),+
        }

        impl TryFrom<u8> for $name {
            type Error = crate::error::Error;
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                use $name::*;
                match value {
                    $($val => Ok($case)),+,
                    _ => Err(ParseError(format!("Unexpected value: {value}"))),
                }
            }
        }

        impl From<$name> for u8 {
            fn from(value: $name) -> u8 {
                use $name::*;
                match value {
                    $($case => $val),+
                }
            }
        }
    };
}

int_enum! {Language,
    English, 0,
    French, 1,
    German, 2,
    Italian, 3,
    Spanish, 4,
    Polish, 5,
    Korean, 128,
    ChineseTraditional, 129,
    ChineseSimplified, 130,
    Japanese, 131
}

fn parse(data: impl Read) {
}
