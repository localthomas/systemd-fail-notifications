/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

use std::collections::HashSet;

use crate::status::UnitStatus;

pub trait SystemdState {
    /// Applies the new state and returns a list of changes compared to the existing state.
    /// Note that the changes are calculated based on the name of the unit.
    fn apply_new_status(&mut self, new_status: Vec<UnitStatus>) -> Vec<ChangedUnitStatus>;
}

pub struct SystemdStateImpl {
    systemd_state: HashSet<UnitStatus>,
}

impl SystemdStateImpl {
    pub fn new() -> Self {
        Self {
            systemd_state: HashSet::new(),
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
    fn apply_new_status(&mut self, new_status: Vec<UnitStatus>) -> Vec<ChangedUnitStatus> {
        let mut new_state = HashSet::with_capacity(new_status.len());
        new_status.into_iter().for_each(|status| {
            new_state.insert(status);
        });
        let old_state = self.systemd_state.clone();
        let changes: Vec<UnitStatus> = new_state.difference(&old_state).cloned().collect();
        self.systemd_state = new_state;

        // map changes: when an item was changed, check if there is an old equivalent (based on the name of the unit) available
        changes
            .into_iter()
            .map(|new_unit_status| {
                let old_unit_status = old_state
                    .iter()
                    .find(|unit_status| unit_status.name() == new_unit_status.name());
                ChangedUnitStatus {
                    old: old_unit_status.cloned(),
                    new: new_unit_status,
                }
            })
            .collect()
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
