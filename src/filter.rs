/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

use crate::{
    state::ChangedUnitStatus,
    status::{ActiveState, LoadState},
};

pub struct FilterState<'a> {
    name_filter: Box<dyn FnMut(&str) -> bool + 'a>,
    state_filter: Box<dyn FnMut(Option<ActiveState>, LoadState, ActiveState) -> bool + 'a>,
}

impl<'a> FilterState<'a> {
    pub fn new() -> Self {
        let name_filter = |name: &str| name.ends_with(".service");
        let state_filter = |old_active_state: Option<ActiveState>,
                            new_load_state: LoadState,
                            new_active_state: ActiveState| match (
            old_active_state,
            new_load_state,
            new_active_state,
        ) {
            (_, LoadState::Error, _) => true,
            // only trigger if not-found is coupled with anything not inactive
            (_, LoadState::NotFound, ActiveState::Inactive) => false,
            (_, LoadState::NotFound, _) => true,
            (_, LoadState::Unknown(_), _) => true,
            (_, _, ActiveState::Failed) => true,
            (_, _, ActiveState::Unknown(_)) => true,
            // otherwise false
            _ => false,
        };
        Self {
            name_filter: Box::new(name_filter),
            state_filter: Box::new(state_filter),
        }
    }

    pub fn filter_function(&mut self, status: &ChangedUnitStatus) -> bool {
        let old_active_state = status.clone().old.map(|state| state.active_state().clone());
        (self.name_filter)(status.new.name())
            && (self.state_filter)(
                old_active_state,
                status.new.load_state().clone(),
                status.new.active_state().clone(),
            )
    }
}
