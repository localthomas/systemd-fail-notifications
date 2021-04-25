/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

use std::collections::HashSet;

use crate::status::UnitStatus;

pub trait SystemdState {
    fn apply_new_status(&mut self, new_status: Vec<UnitStatus>) -> Vec<UnitStatus>;
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

impl<'a> SystemdState for SystemdStateImpl {
    fn apply_new_status(&mut self, new_status: Vec<UnitStatus>) -> Vec<UnitStatus> {
        let mut new_state = HashSet::with_capacity(new_status.len());
        new_status.into_iter().for_each(|status| {
            new_state.insert(status);
        });
        let changes: Vec<UnitStatus> = new_state.difference(&self.systemd_state).cloned().collect();
        self.systemd_state = new_state;
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
        fn apply_new_status(&mut self, new_status: Vec<UnitStatus>) -> Vec<UnitStatus> {
            self.last_state = Some(new_status.clone());
            new_status
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
            vec![test_status.clone()]
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
            vec![test_status.clone()]
        );
    }

    #[test]
    fn on_state_changed_note_called_for_smaller_state() {
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
            vec![test_status.clone()]
        );
        assert_eq!(state.apply_new_status(vec![]), vec![]);
        assert_eq!(state.apply_new_status(vec![]), vec![]);
    }
}
