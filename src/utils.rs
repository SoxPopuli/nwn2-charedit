pub fn pair_second<A, B>((_, b): (A, B)) -> B {
    b
}

#[macro_export]
macro_rules! int_enum {
    ($name: ident, $($case: ident, $val: expr),+) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name {
            $($case = $val),+
        }

        impl $name {
            pub fn as_u8(&self) -> u8 {
                use $name::*;
                match self {
                    $($case => $val),+
                }
            }
        }

        impl TryFrom<u8> for $name {
            type Error = $crate::error::Error;
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                use $name::*;
                use $crate::error::Error::EnumError;
                match value {
                    $($val => Ok($case)),+,
                    _ => Err(EnumError{
                        enum_type: stringify!($name),
                        msg: format!("Unexpected value: {value}")
                    }),
                }
            }
        }

        impl From<$name> for u8 {
            fn from(value: $name) -> u8 {
                value.as_u8()
            }
        }
    };
}
