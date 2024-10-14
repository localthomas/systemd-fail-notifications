/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

use std::collections::HashMap;

use crate::status::UnitStatus;

pub trait SystemdState {
    /// Applies the new state and returns a list of changes compared to the existing state.
    /// Note that the changes are calculated based on the name of the unit.
    /// Note that once a status for a unit name was saved, it is never "forgotten", even if a new state does not contain the unit name any more.
    fn apply_new_status(&mut self, new_status: Vec<UnitStatus>) -> Vec<ChangedUnitStatus>;
}

pub struct SystemdStateImpl {
    systemd_state: HashMap<String, UnitStatus>,
}

impl SystemdStateImpl {
    pub fn new() -> Self {
        Self {
            systemd_state: HashMap::new(),
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
        changes
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

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
        let mut state = SystemdStateImpl::new();
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
        let mut state = SystemdStateImpl::new();
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
        let mut state = SystemdStateImpl::new();
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
        let mut state = SystemdStateImpl::new();
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
        let mut state = SystemdStateImpl::new();
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
