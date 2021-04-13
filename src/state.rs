use std::collections::HashSet;

use crate::status::UnitStatus;

pub trait AppState {
    fn apply_new_status(&mut self, new_status: Vec<UnitStatus>);
}

pub struct SystemdState<'a> {
    systemd_state: HashSet<UnitStatus>,
    // TODO: add filter function as field
    on_state_changed: Box<dyn Fn(&[&UnitStatus]) + 'a>,
}

impl<'a> SystemdState<'a> {
    pub fn new(on_state_changed: impl Fn(&[&UnitStatus]) + 'a) -> Self {
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
}
