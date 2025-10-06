pub mod spell;

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
                    $(Self::$k => write!(f, stringify!($k)),)+
                    $name(x) => write!(f, "{x}"),
                }
            }
        }
    };
}
