/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

use crate::dbus_systemd::dbus::UnitStatusRaw;
use std::str::FromStr;

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

        impl std::fmt::Display for $name {
           fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", match self {
                $(
                    &$name::$item => String::from($repr),
                )*
                    $name::Unknown(string) => format!("unknown: {}", string),
                })
            }
        }

        impl std::str::FromStr for $name {
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

string_enum! {
    ActiveState {
        (Active, "active"),
        (Reloading, "reloading"),
        (Inactive, "inactive"),
        (Failed, "failed"),
        (Activating, "activating"),
        (Deactivating, "deactivating"),
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct UnitStatus {
    name: String,
    description: String,
    load_state: LoadState,
    active_state: ActiveState,
    sub_state: String,
}

impl UnitStatus {
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

    /// Get a reference to the unit status's active state.
    pub fn active_state(&self) -> &ActiveState {
        &self.active_state
    }

    /// Get a reference to the unit status's sub state.
    pub fn sub_state(&self) -> &String {
        &self.sub_state
    }
}

impl From<UnitStatusRaw> for UnitStatus {
    fn from(raw: UnitStatusRaw) -> Self {
        Self {
            name: raw.name,
            description: raw.description,
            load_state: LoadState::from_str(&raw.load_state).unwrap(),
            active_state: ActiveState::from_str(&raw.active_state).unwrap(),
            sub_state: raw.sub_state,
        }
    }
}
