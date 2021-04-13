use std::collections::HashSet;

use crate::status::UnitStatus;

pub trait AppState {
    fn apply_new_status(&mut self, new_status: Vec<UnitStatus>);
}

pub struct SystemdState<'a> {
    systemd_state: HashSet<UnitStatus>,
    // TODO: add filter function as field
    on_state_changed: Box<dyn FnMut(&[&UnitStatus]) + 'a>,
}

impl<'a> SystemdState<'a> {
    pub fn new(on_state_changed: impl FnMut(&[&UnitStatus]) + 'a) -> Self {
        Self {
            systemd_state: HashSet::new(),
            on_state_changed: Box::new(on_state_changed),
        }
    }
}

impl<'a> AppState for SystemdState<'a> {
    fn apply_new_status(&mut self, new_status: Vec<UnitStatus>) {
        // TODO add filter field
        let mut new_state = HashSet::with_capacity(new_status.len());
        new_status
            .into_iter()
            .filter(|status| status.name().ends_with(".service"))
            .for_each(|status| {
                new_state.insert(status);
            });
        let changes: Vec<&UnitStatus> = new_state.difference(&self.systemd_state).collect();
        if changes.len() > 0 {
            (self.on_state_changed)(changes.as_slice());
        }
        self.systemd_state = new_state;
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub struct MockupAppState {
        pub last_state: Option<Vec<UnitStatus>>,
    }

    impl MockupAppState {
        pub fn new() -> Self {
            Self { last_state: None }
        }
    }

    impl AppState for MockupAppState {
        fn apply_new_status(&mut self, new_status: Vec<UnitStatus>) {
            self.last_state = Some(new_status);
        }
    }

    #[test]
    fn on_state_changed_not_called_for_empty_new_state() {
        let mut state = SystemdState::new(|_| panic!("should not be called"));
        state.apply_new_status(vec![]);
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
        let mut counter = 0u32;
        {
            let mut state = SystemdState::new(|new_state| {
                assert_eq!(new_state, vec![&test_status]);
                counter += 1;
            });
            state.apply_new_status(vec![test_status.clone()]);
        }
        assert_eq!(counter, 1);
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
        let mut counter = 0u32;
        {
            let mut state = SystemdState::new(|new_state| {
                assert_eq!(new_state, vec![&test_status]);
                counter += 1;
            });
            state.apply_new_status(vec![]);
            state.apply_new_status(vec![test_status.clone()]);
        }
        assert_eq!(counter, 1);
    }
}
