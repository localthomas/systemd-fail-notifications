use crate::dbus_systemd::dbus::UnitStatusRaw;

/// Generate a string based enum, i.e. one enum variant corresponds to one string value.
/// Does not return an error on parsing, but instead implements a variant `Unknown`.
macro_rules! string_enum {
    ($name:ident { $(($item:ident, $repr:expr),)* }) => {
        #[derive(Debug,  Clone, Eq, PartialEq, Hash)]
        pub enum $name {
            $(
                $item,
            )*
            /// If none of the other variants match, this variant is used.
            Unknown(String),
        }

        use std::fmt;
        impl fmt::Display for $name {
           fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", match self {
                $(
                    &$name::$item => String::from($repr),
                )*
                    LoadState::Unknown(string) => format!("unknown: {}", string),
                })
            }
        }

        use std::str::FromStr;
        impl FromStr for $name {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $repr => Ok($name::$item),
                    )*
                    _ => Ok($name::Unknown(String::from(s))),
                }
            }
        }
    }
}

string_enum! {
    LoadState {
        (Loaded, "loaded"),
        (Error, "error"),
        (Masked, "masked"),
        (NotFound, "not-found"),
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct UnitStatus {
    name: String,
    description: String,
    load_state: LoadState,
}

impl UnitStatus {
    pub fn new(raw: UnitStatusRaw) -> Self {
        Self {
            name: raw.name,
            description: raw.description,
            load_state: LoadState::from_str(&raw.load_state).unwrap(),
        }
    }

    /// Get a reference to the unit status's name.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Get a reference to the unit status's description.
    pub fn description(&self) -> &String {
        &self.description
    }

    /// Get a reference to the unit status's load state.
    pub fn load_state(&self) -> &LoadState {
        &self.load_state
    }
}
