pub mod error;

#[macro_export]
macro_rules! open_enum {
    ($viz: vis enum $name: ident : $repr: ty { $($k: ident = $v: expr),+ $(,)? }) => {
        #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
        #[repr(transparent)]
        $viz struct $name(pub $repr);

        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        impl $name {
            $(pub const $k: $name = $name($v);)+
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    $(Self::$k => f.write_str(stringify!($k)),)+
                    $name(x) => write!(f, "{x}"),
                }
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    $(Self::$k => f.write_str(stringify!($k)),)+
                    $name(x) => write!(f, "{x}"),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! int_enum {
    ($viz: vis enum $name: ident : $sz: ty { $($case: ident = $val: expr),+ $(,)? }) => {
        #[repr($sz)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name {
            $($case = $val),+
        }

        impl $name {
            pub fn as_num(&self) -> $sz {
                use $name::*;
                match self {
                    $($case => $val),+
                }
            }
        }

        impl TryFrom<$sz> for $name {
            type Error = $crate::error::EnumError;
            fn try_from(value: $sz) -> Result<Self, Self::Error> {
                use $name::*;
                use $crate::error::*;
                match value {
                    $($val => Ok($case)),+,
                    _ => Err(EnumError{
                        enum_type: stringify!($name),
                        msg: format!("Unexpected value: {value}")
                    }),
                }
            }
        }

        impl From<$name> for $sz {
            fn from(value: $name) -> $sz {
                value.as_num()
            }
        }
    };
}

