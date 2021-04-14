use crate::status::{ActiveState, LoadState, UnitStatus};

pub struct FilterState<'a> {
    name_filter: Box<dyn FnMut(&str) -> bool + 'a>,
    state_filter: Box<dyn FnMut(&LoadState, &ActiveState) -> bool + 'a>,
}

impl<'a> FilterState<'a> {
    pub fn new() -> Self {
        let name_filter = |name: &str| name.ends_with(".service");
        let state_filter =
            |load_state: &LoadState, active_state: &ActiveState| match (load_state, active_state) {
                (LoadState::Error, _) => true,
                // only trigger if not-found is coupled with anything not inactive
                (LoadState::NotFound, ActiveState::Inactive) => false,
                (LoadState::NotFound, _) => true,
                (LoadState::Unknown(_), _) => true,
                (_, ActiveState::Failed) => true,
                (_, ActiveState::Unknown(_)) => true,
                _ => false,
            };
        Self {
            name_filter: Box::new(name_filter),
            state_filter: Box::new(state_filter),
        }
    }

    pub fn filter_function(&mut self, status: &UnitStatus) -> bool {
        (self.name_filter)(status.name())
            && (self.state_filter)(status.load_state(), status.active_state())
    }
}
