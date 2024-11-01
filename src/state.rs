/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

use std::{collections::HashMap, fs, path::PathBuf};

use crate::status::UnitStatus;

pub trait SystemdState {
    /// Applies the new state and returns a list of changes compared to the existing state.
    /// Note that the changes are calculated based on the name of the unit.
    /// Note that once a status for a unit name was saved, it is never "forgotten", even if a new state does not contain the unit name any more.
    fn apply_new_status(&mut self, new_status: Vec<UnitStatus>) -> Vec<ChangedUnitStatus>;
}

pub struct SystemdStateImpl {
    state_file_path: PathBuf,
    systemd_state: HashMap<String, UnitStatus>,
}

impl SystemdStateImpl {
    pub fn new(state_file_path: PathBuf) -> Self {
        // read state from disk
        let systemd_state = {
            if let Ok(data) = fs::read(&state_file_path) {
                if let Ok(deserialized) = serde_json::from_slice(&data) {
                    deserialized
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            }
        };

        Self {
            state_file_path,
            systemd_state,
        }
    }
}

/// Holds the information about a change between two states.
/// Namely, the old and new states.
#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct ChangedUnitStatus {
    pub old: Option<UnitStatus>,
    pub new: UnitStatus,
}

impl<'a> SystemdState for SystemdStateImpl {
    fn apply_new_status(&mut self, new_state: Vec<UnitStatus>) -> Vec<ChangedUnitStatus> {
        // apply the new state unit by unit and check for changes
        let mut changes: Vec<ChangedUnitStatus> = Vec::new();
        for new_status in new_state {
            // set the new one and get the old one
            let old_status = self
                .systemd_state
                .insert(new_status.name().clone(), new_status.clone());
            // check for a change
            if let Some(old_status) = old_status {
                if old_status != new_status {
                    // change detected: store as change for return value
                    changes.push(ChangedUnitStatus {
                        old: Some(old_status),
                        new: new_status,
                    });
                }
            } else {
                // there was no previous value, this is also a change
                changes.push(ChangedUnitStatus {
                    old: None,
                    new: new_status,
                });
            }
        }

        // save new state to disk
        let serialized_state =
            serde_json::to_string(&self.systemd_state).expect("could not serialize systemd state");
        let state_file_dir_path = self
            .state_file_path
            .parent()
            .expect("could not get parent dir of state_file_path");
        fs::create_dir_all(state_file_dir_path).expect(&format!(
            "could not create directories for the state file ({:?})",
            state_file_dir_path
        ));
        fs::write(&self.state_file_path, serialized_state).expect(&format!(
            "could not write systemd state to file ({:?})",
            self.state_file_path
        ));

        changes
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn temp_file_path() -> PathBuf {
        let mut dir = std::env::temp_dir();
        let file_name: String =
            rand::Rng::sample_iter(rand::thread_rng(), &rand::distributions::Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();
        dir.push(file_name);
        dir
    }

    pub struct MockupSystemdState {
        pub last_state: Option<Vec<UnitStatus>>,
    }

    impl MockupSystemdState {
        pub fn new() -> Self {
            Self { last_state: None }
        }
    }

    impl SystemdState for MockupSystemdState {
        /// Discards the old state completely and returns the new status as is.
        fn apply_new_status(&mut self, new_status: Vec<UnitStatus>) -> Vec<ChangedUnitStatus> {
            self.last_state = Some(new_status.clone());
            new_status
                .into_iter()
                .map(|status| ChangedUnitStatus {
                    old: None,
                    new: status,
                })
                .collect()
        }
    }

    #[test]
    fn on_state_changed_not_called_for_empty_new_state() {
        let mut state = SystemdStateImpl::new(temp_file_path());
        let changes = state.apply_new_status(vec![]);
        assert_eq!(changes, vec![]);
    }

    #[test]
    fn on_state_changed_called_for_new_state() {
        let test_status = UnitStatus::from(crate::dbus_systemd::dbus::UnitStatusRaw {
            name: String::from("name"),
            description: String::from("desc"),
            load_state: String::from("test"),
            active_state: String::from("test"),
            sub_state: String::from("test"),
            following_unit: String::from("test"),
        });
        let mut state = SystemdStateImpl::new(temp_file_path());
        assert_eq!(
            state.apply_new_status(vec![test_status.clone()]),
            vec![ChangedUnitStatus {
                old: None,
                new: test_status.clone(),
            }]
        );
    }

    #[test]
    fn on_state_changed_called_for_changing_state() {
        let test_status = UnitStatus::from(crate::dbus_systemd::dbus::UnitStatusRaw {
            name: String::from("name"),
            description: String::from("desc"),
            load_state: String::from("test"),
            active_state: String::from("test"),
            sub_state: String::from("test"),
            following_unit: String::from("test"),
        });
        let mut state = SystemdStateImpl::new(temp_file_path());
        assert_eq!(state.apply_new_status(vec![]), vec![]);
        assert_eq!(
            state.apply_new_status(vec![test_status.clone()]),
            vec![ChangedUnitStatus {
                old: None,
                new: test_status.clone(),
            }]
        );
    }

    #[test]
    fn on_state_changed_called_for_smaller_state() {
        let test_status = UnitStatus::from(crate::dbus_systemd::dbus::UnitStatusRaw {
            name: String::from("name"),
            description: String::from("desc"),
            load_state: String::from("test"),
            active_state: String::from("test"),
            sub_state: String::from("test"),
            following_unit: String::from("test"),
        });
        let mut state = SystemdStateImpl::new(temp_file_path());
        assert_eq!(
            state.apply_new_status(vec![test_status.clone()]),
            vec![ChangedUnitStatus {
                old: None,
                new: test_status.clone(),
            }]
        );
        assert_eq!(state.apply_new_status(vec![]), vec![]);
        assert_eq!(state.apply_new_status(vec![]), vec![]);
    }

    #[test]
    fn on_state_changed_called_for_one_changed_state() {
        let test_status_old = UnitStatus::from(crate::dbus_systemd::dbus::UnitStatusRaw {
            name: String::from("name"),
            description: String::from("desc"),
            load_state: String::from("test"),
            active_state: String::from("test"),
            sub_state: String::from("test"),
            following_unit: String::from("test"),
        });
        let test_status_new = UnitStatus::from(crate::dbus_systemd::dbus::UnitStatusRaw {
            name: String::from("name"),
            description: String::from("desc"),
            load_state: String::from("test"),
            active_state: String::from("test123"),
            sub_state: String::from("test"),
            following_unit: String::from("test"),
        });
        let mut state = SystemdStateImpl::new(temp_file_path());
        assert_eq!(
            state.apply_new_status(vec![test_status_old.clone()]),
            vec![ChangedUnitStatus {
                old: None,
                new: test_status_old.clone(),
            }]
        );
        assert_eq!(
            state.apply_new_status(vec![test_status_new.clone()]),
            vec![ChangedUnitStatus {
                old: Some(test_status_old),
                new: test_status_new.clone(),
            }]
        );
    }
}
